//! End to end through `AppContext` and the composed services on a synthetic library in a temporary folder, with
//! isolated app data (as `RAGIN_DMS_DATA_DIR` gives), the real storage and PDF adapters, a recording trash and a stub
//! AI. Invented files only; nothing outside the temporary folder is touched.

use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dms_application::classification::{ApprovedCategory, SuggestOutcome, SuggestionApproval};
use dms_application::common::{NoProgress, OperationOutcome};
use dms_application::export::{ExportOptions, ExportOutcome, ExportScope};
use dms_application::library::{ConfigureLibraryOutcome, LibraryCommand};
use dms_application::pdf::SearchableOutcome;
use dms_application::port::ai::{AiServer, ClassificationRequest, ClassificationResult, DocumentClassifier, ModelList};
use dms_application::port::pdf::PageChange;
use dms_application::port::state::Preferences;
use dms_application::port::storage::{CatalogLoad, CatalogRepository};
use dms_application::restructure::{PlanOutcome, RestructureHistoryOutcome, RestructureOutcome, RestructureScope};
use dms_application::services::AppServices;
use dms_domain::catalog::Catalog;
use dms_domain::change::DetectedChange;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{Document, DocumentId};
use dms_domain::library::{FolderLocation, LibraryFolder, RelativePath};
use dms_domain::restructure::FileNaming;
use dms_domain::task::OpenTask;
use dms_infra_pdf::PdfEngine;
use dms_infra_storage::{FileCatalogRepository, SystemTrash};
use indexmap::{IndexMap, IndexSet};

use crate::context::{AppContext, ports};
use crate::session::{
    Lane, LibraryStatus, LogEntry, OperationProgress, Session, SessionEvents, SessionState, UiMessage,
};

const WAIT: Duration = Duration::from_secs(60);

#[test]
#[allow(clippy::too_many_lines)] // one walk through the whole app, step by step
fn a_synthetic_library_goes_through_every_main_flow() {
    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    let cloud = root.path().join("Cloud");
    let outside = root.path().join("Outside");
    for folder in [&primary, &cloud, &outside] {
        std::fs::create_dir_all(folder).expect("folder");
    }
    write_text_pdf(&primary.join("Invoice 2025.pdf"), Some("Invoice 2025 Quick Garage"));
    write_text_pdf(&primary.join("Contract.pdf"), Some("Rental contract Maple Housing"));
    write_text_pdf(&primary.join("scan-0001.pdf"), None);
    write_png(&primary.join("Receipt.png"));
    write_docx(&primary.join("Budget.docx"), "Budget plan");
    std::fs::copy(primary.join("Contract.pdf"), primary.join("Contract copy.pdf")).expect("copy");

    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    let services = app.services();

    // Configure a primary and a sync folder, then open: the scan adopts the new files.
    configure(
        services,
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    configure(services, &LibraryCommand::AddSyncFolder { display_name: "Cloud".into(), location: location(&cloud) });
    app.run(AppContext::open);
    assert!(matches!(app.ctx.session.state().status, LibraryStatus::Open { .. }), "{:?}", app.messages());
    let catalog = app.catalog();
    let names = file_names(&catalog);
    for name in ["Invoice 2025.pdf", "scan-0001.pdf", "Receipt.png", "Budget.docx"] {
        assert!(names.contains(&name.to_owned()), "{name} adopted: {names:?}");
    }
    // Of two files with one content only the first found is adopted; the other waits as a new file.
    let (adopted, waiting) = if names.contains(&"Contract.pdf".to_owned()) {
        ("Contract.pdf", "Contract copy.pdf")
    } else {
        ("Contract copy.pdf", "Contract.pdf")
    };
    assert!(!names.contains(&waiting.to_owned()), "{names:?}");
    let copy = app.new_file(waiting);
    // Files the scan adopted are no new-file tasks on the dashboard; only the copy is.
    let new_file_tasks: Vec<String> = app
        .ctx
        .dashboard()
        .expect("dashboard")
        .tasks
        .into_iter()
        .filter_map(|task| match task {
            OpenTask::AdoptNewFile(file) => Some(file.path.file_name().to_owned()),
            _ => None,
        })
        .collect();
    assert_eq!(new_file_tasks, vec![waiting.to_owned()]);

    // A duplicate pair: adopting the copy anyway gives two documents with one content; keep the first.
    done(services.documents.adopt_new_file.run(&copy));
    app.ctx.forget(&DetectedChange::NewFile(copy));
    let contract = app.document(adopted).id;
    let duplicates = app.ctx.dashboard().expect("dashboard").tasks.into_iter().find_map(|task| match task {
        OpenTask::ResolveDuplicates { document_ids, .. } => Some(document_ids),
        _ => None,
    });
    assert!(duplicates.is_some_and(|ids| ids.contains(&contract)), "a duplicates task");
    done(services.sync.merge_duplicates.run(&contract, true));
    assert!(!file_names(&app.catalog()).contains(&waiting.to_owned()));
    assert_eq!(app.trashed_names(), vec![waiting]);

    // Edit metadata, tags and category.
    let invoice = app.document("Invoice 2025.pdf").id;
    done(
        services
            .documents
            .edit_catalog
            .run(&dms_application::catalog::CatalogEdit::AddCategory { name: "Car".into(), parent_id: None }),
    );
    let car = app.catalog().categories().values().find(|category| category.name == "Car").expect("category").id.clone();
    let metadata =
        dms_domain::document::DocumentMetadata::new(Some("Garage invoice".into()), None, Some("Quick Garage".into()))
            .expect("metadata");
    done(
        services
            .documents
            .edit_catalog
            .run(&dms_application::catalog::CatalogEdit::UpdateMetadata { document_id: invoice.clone(), metadata }),
    );
    done(services.documents.edit_catalog.run(&dms_application::catalog::CatalogEdit::AssignCategory {
        document_id: invoice.clone(),
        category_id: Some(car.clone()),
    }));
    done(services.documents.edit_catalog.run(&dms_application::catalog::CatalogEdit::AddTags {
        document_id: invoice.clone(),
        tags: IndexSet::from([tag("invoice"), tag("car")]),
    }));
    let edited = app.document_by_id(&invoice);
    assert_eq!(edited.metadata.title.as_deref(), Some("Garage invoice"));
    assert_eq!(edited.category_id, Some(car));
    assert_eq!(edited.tags, IndexSet::from([tag("invoice"), tag("car")]));

    // Create a folder, move into it, rename.
    done(services.documents.create_folder.run(&path("Car")));
    assert!(primary.join("Car").is_dir());
    done(services.documents.move_document.run(&invoice, &path("Car/Invoice 2025.pdf")));
    done(services.documents.move_document.run(&invoice, &path("Car/Garage invoice.pdf")));
    assert!(primary.join("Car/Garage invoice.pdf").is_file());
    assert_eq!(app.document_by_id(&invoice).path, path("Car/Garage invoice.pdf"));

    // Trash a document: the recording trash got the file.
    let budget = app.document("Budget.docx").id;
    done(services.documents.trash_document.run(&budget, true));
    assert!(app.trashed_names().contains(&"Budget.docx".to_owned()));
    assert!(!primary.join("Budget.docx").exists());

    // Sync to all: the sync folder mirrors the primary.
    app.run(AppContext::sync_to_all_destinations);
    assert!(cloud.join("Car/Garage invoice.pdf").is_file(), "{:?}", app.messages());
    assert!(cloud.join("dms-catalog.yaml").is_file());

    // Merge: a file only the sync folder has comes to the primary as a new file, and the scan adopts it.
    write_text_pdf(&cloud.join("From phone.pdf"), Some("Photo of a parking ticket"));
    app.run(AppContext::merge_all_folders);
    assert!(primary.join("From phone.pdf").is_file(), "{:?}", app.messages());
    assert!(file_names(&app.catalog()).contains(&"From phone.pdf".to_owned()));

    // Classify with the stub AI, review on the dashboard, approve.
    let scan = app.document("scan-0001.pdf").id;
    let SuggestOutcome::Suggested(_) = services.ai.suggest_classification.run(&scan) else { panic!("a suggestion") };
    app.ctx.session.changed();
    let review = app
        .ctx
        .dashboard()
        .expect("dashboard")
        .tasks
        .into_iter()
        .any(|task| matches!(task, OpenTask::ReviewSuggestion(suggestion) if suggestion.document_id == scan));
    assert!(review, "the suggestion waits for review");
    let approval = SuggestionApproval {
        title: Some("Parking ticket".into()),
        document_date: None,
        correspondent: Some("City".into()),
        category: Some(ApprovedCategory::Path(vec!["Car".into(), "Tickets".into()])),
        tags: IndexSet::from([tag("ticket")]),
        file_name: None,
        target_folder: None,
        fields: IndexMap::new(),
    };
    done(services.ai.approve_suggestion.run(&scan, &approval));
    let approved = app.document_by_id(&scan);
    assert_eq!(approved.metadata.title.as_deref(), Some("Parking ticket"));
    assert!(approved.classification.is_some());

    // PDF tools: a text layer from the AI's transcription, rotate a page, merge two PDFs.
    match services.pdf.make_searchable.run(&scan, false) {
        SearchableOutcome::Made(outcome) => done(outcome),
        other => panic!("a text layer: {other:?}"),
    }
    done(services.pdf.edit_pages.run(&contract, &[PageChange::new(0, 90).expect("quarter turn")]));
    let phone = app.document("From phone.pdf").id;
    done(services.pdf.merge_documents.run(&[contract.clone(), phone.clone()], "Merged.pdf", false));
    let merged = app.document("Merged.pdf");
    let pages = services.pdf.editor.page_count(&app.primary(), &merged.path).expect("page count");
    assert_eq!(pages, 2);

    // Restructure by category into folders, then reverse it.
    let plan = match services.organize.plan_restructure.run(
        &RestructureScope::Folder(RelativePath::root()),
        "{category}",
        FileNaming::Keep,
    ) {
        PlanOutcome::Planned(plan) => plan,
        other => panic!("a plan: {other:?}"),
    };
    let before = app.paths();
    let RestructureOutcome::Applied { moved, record, .. } =
        services.organize.apply_restructure_plan.run(&plan, 0, &NoProgress)
    else {
        panic!("applied")
    };
    assert!(moved > 0, "the plan moved documents");
    assert_ne!(app.paths(), before);
    let record = record.expect("a restructure record");
    let RestructureHistoryOutcome::Listed(history) = services.organize.list_restructure_history.run() else {
        panic!("history")
    };
    assert_eq!(history.first(), Some(&record));
    let RestructureOutcome::Applied { .. } = services.organize.reverse_restructure.run(&record, &NoProgress) else {
        panic!("reversed")
    };
    assert_eq!(app.paths(), before);

    // Export to a ZIP outside the library.
    let ids: Vec<DocumentId> = app.catalog().documents().keys().cloned().collect();
    let options = ExportOptions { zip: true, ..ExportOptions::default() };
    let exported = services.organize.export_documents.run(
        &ExportScope::Documents(ids.clone()),
        &outside.to_string_lossy(),
        "export",
        options,
    );
    let ExportOutcome::Exported { count, location } = exported else { panic!("exported: {exported:?}") };
    assert_eq!(count, ids.len());
    assert!(
        Path::new(&location).is_file() && Path::new(&location).extension().is_some_and(|extension| extension == "zip"),
        "{location}"
    );

    // Sync once more; both folders' catalogs read back through infra-storage and equal the open catalog.
    app.run(AppContext::sync_to_all_destinations);
    let open = app.catalog();
    let repository = FileCatalogRepository::new(Arc::new(RecordingTrash::new(root.path().join("trash-2"))));
    let folders = services.library.settings.load_library_folders().expect("settings").expect("folders");
    assert_eq!(folders.folders().len(), 2);
    for folder in folders.folders() {
        let CatalogLoad::Found { catalog, conflict_copies } = repository.load(folder) else {
            panic!("catalog in {}", folder.display_name)
        };
        assert_eq!(conflict_copies, Vec::<Catalog>::new());
        assert_eq!(catalog.documents(), open.documents(), "documents in {}", folder.display_name);
        assert_eq!(catalog.categories(), open.categories());
    }
    assert!(app.messages().iter().all(|message| !message.is_error), "{:?}", app.messages());
}

/// The composed app on isolated app data, with the stub AI and the recording trash.
struct TestApp {
    ctx: Arc<AppContext>,
    events: Arc<Recorder>,
    trash: Arc<RecordingTrash>,
}

impl TestApp {
    fn start(data_dir: &Path, trash_dir: PathBuf) -> Self {
        let pdfium = PdfEngine::dev_library().expect("PDFium missing: run scripts/fetch-pdfium.sh");
        let trash = Arc::new(RecordingTrash::new(trash_dir));
        let mut ports = ports(data_dir, &pdfium, trash.clone()).expect("ports");
        let ai = Arc::new(StubAi);
        ports.classifier = ai.clone();
        ports.ai_server = ai;
        let services = AppServices::new(ports);
        let ai_on = Preferences {
            ai: dms_application::port::state::AiSettings { enabled: true, ..Default::default() },
            ..Preferences::default()
        };
        services.library.settings.save_preferences(&ai_on).expect("preferences");
        let events = Arc::new(Recorder::default());
        let ctx = Arc::new(AppContext::with_services(Session::new(events.clone()), services));
        Self { ctx, events, trash }
    }

    fn services(&self) -> &AppServices {
        self.ctx.services().expect("services")
    }

    /// Starts a session operation and waits until no operation runs anymore.
    fn run(&self, start: impl FnOnce(&Arc<AppContext>) -> Result<(), crate::dto::IpcError>) {
        start(&self.ctx).expect("the operation starts");
        let deadline = Instant::now() + WAIT;
        while !self.ctx.session.state().operations.is_empty() {
            assert!(Instant::now() < deadline, "the operation finishes");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn catalog(&self) -> Catalog {
        self.ctx.catalog().expect("an open catalog")
    }

    fn document(&self, name: &str) -> Document {
        self.catalog()
            .documents()
            .values()
            .find(|document| document.path.file_name() == name)
            .cloned()
            .unwrap_or_else(|| panic!("{name} in the catalog"))
    }

    fn document_by_id(&self, id: &DocumentId) -> Document {
        self.catalog().documents().get(id).cloned().expect("document")
    }

    fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> =
            self.catalog().documents().values().map(|document| document.path.value().to_owned()).collect();
        paths.sort();
        paths
    }

    fn new_file(&self, name: &str) -> dms_domain::change::NewFile {
        let scan = self.ctx.last_scan().expect("a scan");
        scan.changes
            .into_iter()
            .find_map(|change| match change {
                DetectedChange::NewFile(file) if file.path.file_name() == name => Some(file),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{name} waits as a new file"))
    }

    fn primary(&self) -> LibraryFolder {
        self.services().library.settings.load_library_folders().expect("settings").expect("folders").primary().clone()
    }

    fn trashed_names(&self) -> Vec<String> {
        self.trash
            .entries
            .lock()
            .expect("lock")
            .iter()
            .filter_map(|entry| entry.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect()
    }

    fn messages(&self) -> Vec<UiMessage> {
        self.events.messages.lock().expect("lock").clone()
    }
}

fn configure(services: &AppServices, command: &LibraryCommand) {
    let outcome = services.library.configure_library.run(command);
    assert!(matches!(outcome, ConfigureLibraryOutcome::Configured(_)), "{outcome:?}");
}

#[allow(clippy::needless_pass_by_value)] // takes the outcome straight from the call
fn done(outcome: OperationOutcome) {
    assert!(matches!(outcome, OperationOutcome::Done { .. }), "{outcome:?}");
}

fn file_names(catalog: &Catalog) -> Vec<String> {
    catalog.documents().values().map(|document| document.path.file_name().to_owned()).collect()
}

fn location(folder: &Path) -> FolderLocation {
    FolderLocation::new(folder.to_string_lossy()).expect("location")
}

fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("path")
}

fn tag(raw: &str) -> dms_domain::taxonomy::TagName {
    dms_domain::taxonomy::TagName::of(raw).expect("tag")
}

/// Classifies every document as a scan of a parking ticket, with a transcription for the text layer.
struct StubAi;

impl DocumentClassifier for StubAi {
    fn suggest(&self, request: &ClassificationRequest) -> ClassificationResult {
        let mut suggestion = ClassificationSuggestion::empty(request.document_id.clone(), "stub-model");
        suggestion.title = Some("Parking ticket".into());
        suggestion.transcription = Some("Parking ticket 12 EUR".into());
        ClassificationResult::Suggested(suggestion)
    }
}

impl AiServer for StubAi {
    fn models(&self, _: &dms_application::port::state::AiSettings) -> ModelList {
        ModelList::Available(vec!["stub-model".into()])
    }
}

/// Moves trashed entries into its own folder and records them.
struct RecordingTrash {
    folder: PathBuf,
    entries: Mutex<Vec<PathBuf>>,
}

impl RecordingTrash {
    fn new(folder: PathBuf) -> Self {
        Self { folder, entries: Mutex::new(Vec::new()) }
    }
}

impl SystemTrash for RecordingTrash {
    fn move_to_trash(&self, _library_root: &Path, entry: &Path) -> bool {
        let mut entries = self.entries.lock().expect("lock");
        let target = self.folder.join(entries.len().to_string());
        let moved = std::fs::create_dir_all(&target).is_ok()
            && entry.file_name().is_some_and(|name| std::fs::rename(entry, target.join(name)).is_ok());
        if moved {
            entries.push(entry.to_path_buf());
        }
        moved
    }
}

#[derive(Default)]
struct Recorder {
    messages: Mutex<Vec<UiMessage>>,
}

impl SessionEvents for Recorder {
    fn state(&self, _: &SessionState) {}
    fn progress(&self, _: &HashMap<Lane, OperationProgress>) {}
    fn log(&self, _: &LogEntry) {}
    fn message(&self, message: &UiMessage) {
        self.messages.lock().expect("lock").push(message.clone());
    }
}

/// A one-page A4 PDF; with `text` in Helvetica, without it a page that holds no text, like a scan.
fn write_text_pdf(file: &Path, text: Option<&str>) {
    let content = text.map_or_else(String::new, |text| format!("BT /F1 12 Tf 50 700 Td ({text}) Tj ET"));
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes(),
    );
    std::fs::write(file, pdf).expect("write PDF");
}

fn write_png(file: &Path) {
    image::RgbImage::from_fn(40, 30, |x, y| {
        image::Rgb([u8::try_from(x * 6).unwrap_or(0), u8::try_from(y * 8).unwrap_or(0), 120])
    })
    .save(file)
    .expect("write PNG");
}

fn write_docx(file: &Path, text: &str) {
    let mut zip = zip::ZipWriter::new(std::fs::File::create(file).expect("create DOCX"));
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", options).expect("entry");
    zip.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#).expect("write");
    zip.start_file("word/document.xml", options).expect("entry");
    let xml = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:body></w:document>"#
    );
    zip.write_all(xml.as_bytes()).expect("write");
    zip.finish().expect("finish DOCX");
}

/// Prepares a synthetic library for a manual run of the app: `DMS_SYNTHETIC_ROOT=<scratch dir> cargo test -p
/// ragin-dms-desktop prepare_synthetic_library -- --ignored`, then start the app with
/// `RAGIN_DMS_DATA_DIR=<scratch dir>/app-data`. Refuses to run without the variable, so it never touches real data.
#[test]
#[ignore = "manual: prepares a scratch library for starting the app"]
fn prepare_synthetic_library() {
    let root =
        PathBuf::from(std::env::var_os("DMS_SYNTHETIC_ROOT").expect("DMS_SYNTHETIC_ROOT names a scratch folder"));
    let primary = root.join("Documents");
    let cloud = root.join("Cloud");
    std::fs::create_dir_all(primary.join("Inbox")).expect("folder");
    std::fs::create_dir_all(&cloud).expect("folder");
    write_text_pdf(&primary.join("Invoice 2025.pdf"), Some("Invoice 2025 Quick Garage"));
    write_text_pdf(&primary.join("Contract.pdf"), Some("Rental contract Maple Housing"));
    write_text_pdf(&primary.join("Inbox/scan-0001.pdf"), None);
    write_png(&primary.join("Receipt.png"));
    write_docx(&primary.join("Budget.docx"), "Budget plan");
    let app = TestApp::start(&root.join("app-data"), root.join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    configure(
        app.services(),
        &LibraryCommand::AddSyncFolder { display_name: "Cloud".into(), location: location(&cloud) },
    );
}
