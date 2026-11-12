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
use dms_application::port::assistant::{
    QueryAnswer, QueryRequest, QueryTranslator, SplitAnswer, SplitProposer, SplitRequest, TranslatedQuery,
};
use dms_application::port::pdf::PageChange;
use dms_application::port::state::Preferences;
use dms_application::port::storage::{CatalogLoad, CatalogRepository};
use dms_application::preview::ThumbnailOutcome;
use dms_application::restructure::{PlanOutcome, RestructureHistoryOutcome, RestructureOutcome, RestructureScope};
use dms_application::services::AppServices;
use dms_domain::catalog::Catalog;
use dms_domain::change::DetectedChange;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::{Document, DocumentId};
use dms_domain::library::{FolderLocation, LibraryFolder, RelativePath};
use dms_domain::restructure::FileNaming;
use dms_domain::split::SplitPart;
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
    write_noise_png(&primary.join("Photo.png"));
    // Search texts and Ask index of version 1.x, flat in their hidden folders.
    std::fs::create_dir_all(primary.join(".dms-text")).expect("folder");
    std::fs::write(primary.join(".dms-text/aa11.txt"), "text of 1.x").expect("flat text");
    std::fs::create_dir_all(cloud.join(".dms-index")).expect("folder");
    std::fs::write(cloud.join(".dms-index/bb22.vec"), "index of 1.x").expect("flat index");

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
    // Opening moved the flat files into subfolders before the scan and marked both folders.
    assert_eq!(std::fs::read_to_string(primary.join(".dms-text/aa/aa11.txt")).expect("moved text"), "text of 1.x");
    assert_eq!(std::fs::read_to_string(cloud.join(".dms-index/bb/bb22.vec")).expect("moved index"), "index of 1.x");
    assert!(!primary.join(".dms-text/aa11.txt").exists() && !cloud.join(".dms-index/bb22.vec").exists());
    for folder in [&primary, &cloud] {
        assert_eq!(std::fs::read_to_string(folder.join(".dms-text/.layout")).expect("marker"), "2\n");
    }
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

    // A preview small against its document is stored in every folder for the content hash; the small receipt's
    // preview would be a tenth of its size or more, so it stays in memory only.
    let photo = app.document("Photo.png");
    let receipt = app.document("Receipt.png");
    for document in [&photo, &receipt] {
        let outcome = services.library.thumbnails.run(&app.primary().id, &document.path);
        assert!(matches!(outcome, ThumbnailOutcome::Rendered(_)), "{outcome:?}");
    }
    let preview_of = |document: &Document, folder: &Path| {
        let name = format!("{}.{}.jpg", document.id.value(), &document.content_hash.value()[..12]);
        sidecar_folder(folder, &document.id).join(name)
    };
    for folder in [&primary, &cloud] {
        assert!(preview_of(&photo, folder).is_file(), "photo preview in {}", folder.display());
        assert!(!preview_of(&receipt, folder).exists(), "no receipt preview in {}", folder.display());
    }

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
        tax: None,
    };
    done(services.ai.approve_suggestion.run(&scan, &approval));
    let approved = app.document_by_id(&scan);
    assert_eq!(approved.metadata.title.as_deref(), Some("Parking ticket"));
    for folder in [&primary, &cloud] {
        let text = sidecar_folder(folder, &scan).join(format!("{}.txt", scan.value()));
        assert!(text.is_file(), "search text in its subfolder in {}", folder.display());
    }
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

/// The 2.1 catalog parts survive saving and opening the library again: the owner's details from the settings card,
/// and a contract, a case and a tax year saved through the catalog store.
#[test]
fn owner_details_and_records_survive_a_save_and_a_reload() {
    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    write_text_pdf(&primary.join("Water bill.pdf"), Some("Water bill Example Water Works"));
    let data_dir = root.path().join("app-data");
    let app = TestApp::start(&data_dir, root.path().join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let bill = app.document("Water bill.pdf");

    let typed = crate::features::settings::dto::SettingsOwnerDto {
        name: "Alex Example".to_owned(),
        address: "Main Street 1\n12345 Exampletown".to_owned(),
        email: "alex@example.org".to_owned(),
        phone: String::new(),
    };
    let crate::features::settings::library_settings::SettingsPlan::Change(change) =
        crate::features::settings::owner::plan_owner(&typed)
    else {
        panic!("the details are valid")
    };
    done(app.services().documents.change_settings.run(&change.subject, |settings| (change.apply)(settings)));
    let mut contract = dms_domain::contract::Contract::new(
        dms_domain::contract::ContractId::new("contract-1").expect("id"),
        "Water",
        dms_domain::contract::ContractKind::Utility,
    )
    .expect("contract");
    contract.document_ids.insert(bill.id.clone());
    let case = dms_domain::cases::Case::new(
        dms_domain::cases::CaseId::new("case-1").expect("id"),
        "Objection",
        chrono::NaiveDate::from_ymd_opt(2026, 10, 1).expect("date"),
    )
    .expect("case");
    let store = &app.services().library.store;
    done(store.commit(|catalog| catalog.save_contract(contract.clone())));
    done(store.commit(|catalog| catalog.save_case(case.clone())));
    done(store.commit(|catalog| catalog.save_tax_year(dms_domain::tax::TaxYear::new(2025))));
    let saved = app.catalog();
    drop(app);

    let reopened = TestApp::start(&data_dir, root.path().join("trash"));
    reopened.run(AppContext::open);
    let catalog = reopened.catalog();
    let owner = catalog.settings().owner.clone().expect("the owner's details");
    assert_eq!(owner.name, "Alex Example");
    assert_eq!(owner.address_lines, vec!["Main Street 1".to_owned(), "12345 Exampletown".to_owned()]);
    assert_eq!(owner.email.as_deref(), Some("alex@example.org"));
    assert_eq!(catalog.contracts(), saved.contracts());
    assert_eq!(catalog.contracts()[0].document_ids.iter().collect::<Vec<_>>(), vec![&bill.id]);
    assert_eq!(catalog.cases(), saved.cases());
    assert_eq!(catalog.tax_years().keys().collect::<Vec<_>>(), vec![&2025]);
    assert!(reopened.messages().iter().all(|message| !message.is_error), "{:?}", reopened.messages());
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
        ports.split_proposer = ai.clone();
        ports.query_translator = ai.clone();
        ports.ai_server = ai;
        let services = AppServices::new(ports);
        // No archive check starts on its own after opening; the health test starts it.
        let ai_on = Preferences {
            ai: dms_application::port::state::AiSettings { enabled: true, ..Default::default() },
            health_check: dms_domain::health::HealthSchedule::Off,
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
        if request.current_file_name.contains("request") {
            suggestion.title = Some("Request for the invoice".into());
            suggestion.requested_actions = vec![dms_domain::classification::RequestedAction {
                text: "Send the invoice".into(),
                due_date: Some(chrono::Local::now().date_naive()),
                references: vec!["Invoice 2025".into()],
            }];
        }
        ClassificationResult::Suggested(suggestion)
    }
}

impl SplitProposer for StubAi {
    fn read_page(&self, _: &dms_application::port::ai::PageImage) -> dms_application::port::ai::LineOutcome {
        dms_application::port::ai::LineOutcome::Read("stub page".into())
    }

    /// The first page is a parking ticket, the rest one document the AI says nothing about.
    fn propose(&self, request: &SplitRequest) -> SplitAnswer {
        let ticket = SplitPart { title: Some("Parking ticket".into()), ..SplitPart::pages(0, 0) };
        SplitAnswer::Proposed {
            parts: vec![ticket, SplitPart::pages(1, request.pages.len() - 1)],
            model: "stub-model".into(),
        }
    }
}

impl QueryTranslator for StubAi {
    /// Names the one known correspondent and a sender the library does not know.
    fn translate(&self, request: &QueryRequest) -> QueryAnswer {
        let query = TranslatedQuery {
            correspondent: request.correspondents.first().cloned(),
            tags: vec!["Unknown Sender GmbH".into()],
            date_from: chrono::NaiveDate::from_ymd_opt(2025, 1, 1),
            date_to: chrono::NaiveDate::from_ymd_opt(2025, 12, 31),
            ..TranslatedQuery::default()
        };
        QueryAnswer::Translated { query, model: "stub-model".into() }
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

/// A photo-like picture: noise compresses badly, so the file is large against its preview.
fn write_noise_png(file: &Path) {
    let mut seed: u32 = 7;
    image::RgbImage::from_fn(800, 600, |_, _| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let [red, green, blue, _] = seed.to_be_bytes();
        image::Rgb([red, green, blue])
    })
    .save(file)
    .expect("write PNG");
}

/// `.dms-text/<xx>/` of a document in `folder`.
fn sidecar_folder(folder: &Path, document_id: &DocumentId) -> PathBuf {
    folder.join(".dms-text").join(document_id.value()[..2].to_lowercase())
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

/// Feature 3: last year's tax documents put their senders on this year's checklist; assigning a document ticks its
/// item; the export for the tax adviser is grouped by area and starts with the missing-items page; it all survives a
/// reload.
#[test]
fn a_tax_year_is_prepared_ticked_off_and_sent_to_the_tax_adviser() {
    use dms_application::catalog::CatalogEdit;
    use dms_domain::document::DocumentMetadata;
    use dms_domain::tax::{TaxArea, TaxAssignment, TaxItemState};

    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    let outside = root.path().join("Exports");
    std::fs::create_dir_all(&primary).expect("folder");
    std::fs::create_dir_all(&outside).expect("folder");
    write_text_pdf(&primary.join("Sweep 2024.pdf"), Some("Chimney sweep invoice 2024"));
    write_text_pdf(&primary.join("Sweep 2025.pdf"), Some("Chimney sweep invoice 2025"));
    let data_dir = root.path().join("app-data");
    let app = TestApp::start(&data_dir, root.path().join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let organize = &app.services().organize;
    let assign = |name: &str, year: i32| {
        let document = app.document(name);
        let metadata =
            DocumentMetadata { correspondent: Some("Chimney Sweep".to_owned()), ..document.metadata.clone() };
        done(
            app.services()
                .documents
                .edit_catalog
                .run(&CatalogEdit::UpdateMetadata { document_id: document.id.clone(), metadata }),
        );
        let tax = TaxAssignment { year, area: TaxArea::HouseholdServices, deductible: None };
        done(organize.assign_tax.run(std::slice::from_ref(&document.id), Some(&tax), true));
        document.id
    };

    assign("Sweep 2024.pdf", 2024);
    done(organize.open_tax_year.run(2025));
    let listed = app.catalog().tax_years()[&2025].clone();
    let sweep = listed.items.iter().find(|item| item.label == "Chimney Sweep").expect("last year's sender is listed");
    assert_ne!(sweep.state, TaxItemState::Found);
    let this_year = assign("Sweep 2025.pdf", 2025);
    let ticked = app.catalog().tax_years()[&2025].items.iter().find(|item| item.id == sweep.id).cloned().expect("item");
    assert_eq!((ticked.state, ticked.document_id), (TaxItemState::Found, Some(this_year)));

    let query = dms_domain::query::DocumentQuery {
        tax: Some(dms_domain::tax::TaxFilter { year: Some(2025), ..Default::default() }),
        ..Default::default()
    };
    let options = ExportOptions { zip: true, tax_year: Some(2025), ..ExportOptions::default() };
    let outcome =
        organize.export_documents.run(&ExportScope::Query(query), &outside.to_string_lossy(), "Tax 2025", options);
    let ExportOutcome::Exported { location, .. } = outcome else { panic!("exported: {outcome:?}") };
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&location).expect("zip")).expect("archive");
    let names: Vec<String> = archive.file_names().map(|name| name.expect("name").into_owned()).collect();
    assert!(names.contains(&"Household services and craftsmen/Sweep 2025.pdf".to_owned()), "{names:?}");
    assert!(names.contains(&"Missing items.csv".to_owned()), "{names:?}");
    let mut page = Vec::new();
    std::io::Read::read_to_end(&mut archive.by_name("Missing items.pdf").expect("the missing-items page"), &mut page)
        .expect("read");
    let page_file = root.path().join("page.pdf");
    std::fs::write(&page_file, page).expect("write");
    let engine = PdfEngine::bind(&PdfEngine::dev_library().expect("PDFium")).expect("bound");
    let texts = engine.page_texts(&page_file).expect("the page opens");
    assert!(
        texts[0].contains("Tax year 2025: missing items") && texts[0].contains("Lohnsteuerbescheinigung"),
        "{texts:?}"
    );
    let saved = app.catalog();
    drop(app);

    let reopened = TestApp::start(&data_dir, root.path().join("trash"));
    reopened.run(AppContext::open);
    assert_eq!(reopened.catalog().tax_years(), saved.tax_years());
    assert_eq!(reopened.document("Sweep 2025.pdf").tax.map(|tax| tax.year), Some(2025));
}

/// Feature 1: a Factur-X PDF becomes a bill without the AI, with the exact values marked as from the e-invoice; once
/// overdue it asks on the dashboard and gets a payment code; marking it paid ends the task, undo brings it back.
#[test]
fn an_e_invoice_becomes_a_bill_to_pay_without_the_ai() {
    use crate::features::bills::dto::{BillSourceDto, BillsFilterDto};
    use crate::features::bills::view;

    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    let today = app.services().library.clock.today();
    let due = today - chrono::Days::new(2);
    write_e_invoice_pdf(&primary.join("Invoice.pdf"), due);
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let bill = app.document("Invoice.pdf");

    let SuggestOutcome::Suggested(suggestion) = app.services().organize.suggest_from_e_invoice.run(&bill.id) else {
        panic!("the e-invoice makes a suggestion")
    };
    assert_eq!(suggestion.model, ClassificationSuggestion::E_INVOICE_MODEL);
    assert_eq!(suggestion.correspondent.as_deref(), Some("Sample Energy GmbH"));
    let approval = SuggestionApproval {
        correspondent: suggestion.correspondent.clone(),
        fields: suggestion.fields.clone(),
        ..SuggestionApproval::default()
    };
    done(app.services().ai.approve_suggestion.run(&bill.id, &approval));
    app.ctx.session.changed();

    let catalog = app.catalog();
    let open = view::bills_view(&catalog, today, BillsFilterDto::Open);
    assert_eq!(open.rows.len(), 1);
    assert_eq!(open.rows[0].source, BillSourceDto::EInvoice);
    assert_eq!(open.rows[0].amount, "86.40 EUR");
    let (code, payment) = view::payment_code(&catalog, &app.document("Invoice.pdf"), today);
    assert_eq!(code.refusal, None);
    assert!(payment.expect("a payment").payload().contains("DE89370400440532013000"));
    let overdue = |app: &TestApp| {
        app.ctx.dashboard().expect("a dashboard").tasks.iter().any(|task| matches!(task, OpenTask::BillOverdue { .. }))
    };
    assert!(overdue(&app));

    done(app.services().organize.mark_bill_paid.run(&bill.id, today));
    app.ctx.session.changed();
    assert!(!overdue(&app));
    assert_eq!(view::bills_view(&app.catalog(), today, BillsFilterDto::Paid).rows.len(), 1);
    done(app.services().organize.mark_bill_unpaid.run(&bill.id));
    app.ctx.session.changed();
    assert!(overdue(&app));
}

/// Feature 5: monthly documents of one sender with the latest one missing ask on the dashboard; "Requested" hides
/// the task for 14 days.
#[test]
fn a_missing_monthly_document_is_expected_until_it_is_requested() {
    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    for month in 0..4 {
        write_text_pdf(&primary.join(format!("slip-{month}.pdf")), Some(&format!("Salary slip {month}")));
    }
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let last = app.services().library.clock.today() - chrono::Days::new(45);
    for month in 0..4u64 {
        let slip = app.document(&format!("slip-{month}.pdf"));
        let metadata = dms_domain::document::DocumentMetadata {
            title: None,
            document_date: Some(last - chrono::Days::new(30 * month)),
            correspondent: Some("Employer Ltd".to_owned()),
        };
        done(app.services().library.store.commit(|catalog| catalog.update_metadata(&slip.id, metadata.clone())));
    }
    app.ctx.session.changed();
    let catalog = app.catalog();
    let tasks = app.ctx.dashboard().expect("a dashboard").tasks;
    let cards = crate::features::bills::view::expected_documents(&tasks, &catalog);
    assert_eq!(cards.len(), 1, "{cards:?}");
    assert_eq!(cards[0].correspondent, "Employer Ltd");
    assert!(cards[0].rhythm.starts_with("Arrived every month since"));

    let key = dms_domain::task::TaskKey::new(cards[0].task_key.clone());
    app.services().organize.snooze_expected_document.run(&key).expect("snoozed");
    app.ctx.session.changed();

    let tasks = app.ctx.dashboard().expect("a dashboard").tasks;
    assert_eq!(crate::features::bills::view::expected_documents(&tasks, &catalog), Vec::new());
}

/// A one-page PDF carrying a Factur-X MINIMUM-like invoice (with payment means) as embedded file `factur-x.xml`.
fn write_e_invoice_pdf(file: &Path, due: chrono::NaiveDate) {
    let xml = format!(
        r#"<rsm:CrossIndustryInvoice xmlns:rsm="r" xmlns:ram="a" xmlns:udt="u"><rsm:ExchangedDocument><ram:ID>RE-1</ram:ID><ram:IssueDateTime><udt:DateTimeString format="102">20260101</udt:DateTimeString></ram:IssueDateTime></rsm:ExchangedDocument><rsm:SupplyChainTradeTransaction><ram:ApplicableHeaderTradeAgreement><ram:SellerTradeParty><ram:Name>Sample Energy GmbH</ram:Name></ram:SellerTradeParty></ram:ApplicableHeaderTradeAgreement><ram:ApplicableHeaderTradeSettlement><ram:InvoiceCurrencyCode>EUR</ram:InvoiceCurrencyCode><ram:SpecifiedTradeSettlementPaymentMeans><ram:PayeePartyCreditorFinancialAccount><ram:IBANID>DE89370400440532013000</ram:IBANID></ram:PayeePartyCreditorFinancialAccount></ram:SpecifiedTradeSettlementPaymentMeans><ram:SpecifiedTradePaymentTerms><ram:DueDateDateTime><udt:DateTimeString format="102">{}</udt:DateTimeString></ram:DueDateDateTime></ram:SpecifiedTradePaymentTerms><ram:SpecifiedTradeSettlementHeaderMonetarySummation><ram:DuePayableAmount>86.40</ram:DuePayableAmount></ram:SpecifiedTradeSettlementHeaderMonetarySummation></ram:ApplicableHeaderTradeSettlement></rsm:SupplyChainTradeTransaction></rsm:CrossIndustryInvoice>"#,
        due.format("%Y%m%d")
    );
    let content = "BT /F1 12 Tf 50 700 Td (Invoice RE-1) Tj ET";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R /Names << /EmbeddedFiles << /Names [(factur-x.xml) 6 0 R] >> >> >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        "<< /Type /Filespec /F (factur-x.xml) /UF (factur-x.xml) /EF << /F 7 0 R >> >>".to_owned(),
        format!("<< /Type /EmbeddedFile /Subtype /text#2Fxml /Length {} >>\nstream\n{xml}\nendstream", xml.len()),
    ];
    write_pdf_objects(file, &objects);
}

/// A PDF of `objects` (numbered from 1, the first one the catalog) with a correct cross-reference table.
fn write_pdf_objects(file: &Path, objects: &[String]) {
    let mut pdf = b"%PDF-1.7\n".to_vec();
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

#[test]
fn a_contract_from_a_proposal_gets_its_cancel_by_task_thirty_days_ahead() {
    use dms_application::library::DashboardOutcome;
    use dms_domain::field::{FieldPresets, FieldValue};
    use dms_domain::settings::CatalogSettings;

    use crate::features::contracts::{editor, view};
    use crate::features::dashboard::dto::DashboardAction;
    use crate::features::dashboard::{plan, run};

    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    write_text_pdf(&primary.join("Gym membership.pdf"), Some("Membership Example Gym"));
    write_text_pdf(&primary.join("Streaming.pdf"), Some("Example Streaming"));
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let today = app.services().library.clock.today();
    let in_days = |days| today + chrono::Days::new(days);
    let store = &app.services().library.store;
    done(store.commit(|catalog| {
        catalog
            .change_settings(CatalogSettings { field_definitions: FieldPresets::all(), ..catalog.settings().clone() })
    }));
    for (name, correspondent) in [("Gym membership.pdf", "Example Gym"), ("Streaming.pdf", "Example Streaming")] {
        let document = app.document(name);
        let metadata = dms_domain::document::DocumentMetadata {
            correspondent: Some(correspondent.to_owned()),
            ..document.metadata.clone()
        };
        let cancel_by = indexmap::indexmap! { FieldPresets::cancel_by().key => FieldValue::Date(in_days(20)) };
        done(store.commit(|catalog| catalog.update_metadata(&document.id, metadata.clone())));
        done(store.commit(|catalog| catalog.set_fields(&document.id, cancel_by.clone())));
    }
    let dashboard = || match app.services().library.build_dashboard.run(None) {
        DashboardOutcome::Built(dashboard) => dashboard,
        DashboardOutcome::LibraryNotOpen => panic!("a library is open"),
    };
    let proposals = |tasks: &[OpenTask]| -> Vec<String> {
        tasks
            .iter()
            .filter_map(|task| match task {
                OpenTask::ReviewContractProposal(proposal) => Some(proposal.correspondent.clone()),
                _ => None,
            })
            .collect()
    };
    assert_eq!(proposals(&dashboard().tasks), vec!["Example Gym".to_owned(), "Example Streaming".to_owned()]);

    // "Dismiss" on the streaming proposal goes through the dashboard's plan.
    let built = dashboard();
    let key = built
        .tasks
        .iter()
        .find(|task| matches!(task, OpenTask::ReviewContractProposal(proposal) if proposal.correspondent == "Example Streaming"))
        .map(|task| task.key().value().to_owned())
        .expect("the streaming proposal");
    let action_plan = plan::plan(&DashboardAction::DismissContractProposal { task_key: key }, &built, &app.catalog())
        .expect("a plan");
    let plan::Work::One { call, .. } = action_plan.work else { panic!("one call") };
    done(run::outcome_of(app.services(), &call));

    // "Create contract" on the gym proposal: the editor filled with it, then price, term and notice entered.
    let gym = app.document("Gym membership.pdf");
    let form = crate::features::contracts::dto::ContractEditorDto {
        price: "34,90 EUR".to_owned(),
        term_end: dms_domain::iso_date(in_days(50)),
        renews_by_months: "12".to_owned(),
        notice_count: "30".to_owned(),
        notice_unit: crate::features::contracts::dto::ContractNoticeUnitDto::Days,
        ..editor::new_form("Example Gym", vec![gym.id.to_string()])
    };
    let draft = editor::draft_of(&form, None).expect("a valid form");
    let saved = app.services().organize.save_contract.run(&draft);
    done(saved.outcome);

    let tasks = dashboard().tasks;
    assert_eq!(proposals(&tasks), Vec::<String>::new());
    assert!(tasks.contains(&OpenTask::ContractCancelBy { contract_id: saved.contract_id.clone(), date: in_days(20) }));
    assert!(
        !tasks.iter().any(|task| matches!(task, OpenTask::DeadlineDue { document_id, .. } if *document_id == gym.id)),
        "the contract's task replaces the field's"
    );
    let card = view::contracts_view(&app.catalog(), today).cards.into_iter().next().expect("the gym card");
    assert_eq!(card.monthly.as_deref(), Some("34.90 EUR"));
    assert_eq!(card.detail.cancel_by, Some(dms_domain::iso_date(in_days(20))));
    assert!(card.pills.iter().any(|pill| pill.label == format!("Cancel by {}", dms_domain::iso_date(in_days(20)))));
}

#[test]
fn a_letter_asking_for_something_becomes_a_case_whose_follow_up_and_waiting_show_on_the_dashboard() {
    use crate::features::cases::commands::apply_review_choice;
    use crate::features::cases::dto::CasesReviewChoiceDto;
    use dms_domain::cases::{CaseStatus, FollowUpSource};

    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    write_text_pdf(&primary.join("tax request.pdf"), Some("Please send us Invoice 2025 by today"));
    write_text_pdf(&primary.join("Invoice 2025.pdf"), Some("Invoice 2025 Quick Garage"));
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    let services = app.services();
    configure(
        services,
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let letter = app.document("tax request.pdf");
    let invoice = app.document("Invoice 2025.pdf");

    // The AI reads the request; the review proposes a follow-up on the requested day and the matching invoice.
    let SuggestOutcome::Suggested(suggestion) = services.ai.suggest_classification.run(&letter.id) else {
        panic!("a suggestion")
    };
    let today = services.library.clock.today();
    let dms_application::search::SearchOutcome::Found(matches) = services.library.search_contents.run("Invoice 2025")
    else {
        panic!("a search")
    };
    let matches: Vec<DocumentId> = matches.into_iter().collect();
    let review =
        crate::features::cases::view::review_cases(&app.catalog(), &letter, Some(&suggestion), &matches, today);
    assert_eq!(review.requested.len(), 1);
    assert_eq!(review.requested[0].date, dms_domain::iso_date(today));
    assert_eq!(
        review.matches.iter().map(|matched| matched.document_id.clone()).collect::<Vec<_>>(),
        vec![invoice.id.to_string()]
    );

    // Approving with "Create a follow-up" starts the case; the dashboard shows the follow-up today.
    let approval = SuggestionApproval {
        title: Some("Request for the invoice".into()),
        document_date: None,
        correspondent: Some("Tax office".into()),
        category: None,
        tags: IndexSet::new(),
        file_name: None,
        target_folder: None,
        fields: IndexMap::new(),
        tax: None,
    };
    done(services.ai.approve_suggestion.run(&letter.id, &approval));
    let choice =
        CasesReviewChoiceDto::NewCase { title: review.new_title.clone(), follow_ups: review.requested.clone() };
    done(apply_review_choice(services, &app.catalog(), &letter.id, &choice));
    app.ctx.session.changed();
    let catalog = app.catalog();
    let case = catalog.cases().values().next().expect("a case").clone();
    assert_eq!((case.title.as_str(), case.correspondent.as_deref()), ("Request for the invoice", Some("Tax office")));
    assert_eq!(case.follow_ups[0].source, FollowUpSource::Ai { document_id: letter.id.clone() });
    let case_tasks = || -> Vec<OpenTask> {
        app.ctx
            .dashboard()
            .expect("dashboard")
            .tasks
            .into_iter()
            .filter(|task| matches!(task, OpenTask::FollowUpDue { .. } | OpenTask::CaseWaitingTooLong { .. }))
            .collect()
    };
    assert_eq!(
        case_tasks(),
        vec![OpenTask::FollowUpDue {
            case_id: case.id.clone(),
            follow_up_id: case.follow_ups[0].id.clone(),
            date: today
        }]
    );

    // Done with the follow-up and sent the invoice: the case waits; 21 days without an answer bring it back.
    done(services.cases.complete_follow_up.run(&case.id, &case.follow_ups[0].id));
    done(services.cases.add_to_case.run(&case.id, &invoice.id, dms_domain::cases::EntryDirection::Sent));
    done(services.cases.set_case_status.run(&case.id, CaseStatus::Waiting));
    app.ctx.session.changed();
    assert_eq!(case_tasks(), Vec::new());
    let since = today - chrono::Days::new(21);
    let waiting = app.catalog().cases()[&case.id].clone();
    let store = &services.library.store;
    done(store.commit(|catalog| {
        catalog.save_case(dms_domain::cases::Case { waiting_since: Some(since), ..waiting.clone() })
    }));
    app.ctx.session.changed();
    assert_eq!(case_tasks(), vec![OpenTask::CaseWaitingTooLong { case_id: case.id.clone(), since }]);
}

/// Archive health: a sync copy altered in place (same size, modification time kept) is found by the check as damaged
/// with the primary as repair source; the dashboard's repair puts the bad copy in the trash. Restoring a catalog
/// snapshot raises the revision and brings the old content back.
#[test]
fn a_damaged_copy_is_found_and_repaired_and_a_snapshot_restores_the_catalog() {
    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    let cloud = root.path().join("Cloud");
    for folder in [&primary, &cloud] {
        std::fs::create_dir_all(folder).expect("folder");
    }
    write_text_pdf(&primary.join("Policy.pdf"), Some("Insurance policy Example Mutual"));
    std::fs::copy(primary.join("Policy.pdf"), cloud.join("Policy.pdf")).expect("copy");
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    let services = app.services();
    configure(
        services,
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    configure(services, &LibraryCommand::AddSyncFolder { display_name: "Cloud".into(), location: location(&cloud) });
    app.run(AppContext::open);
    let policy = app.document("Policy.pdf");
    let folders = services.library.settings.load_library_folders().expect("settings").expect("folders");
    let (primary_id, cloud_id) = (folders.primary().id.clone(), folders.sync_folders()[0].id.clone());
    let snapshot_before = services.sync.snapshots.list(app.catalog().library_id()).expect("snapshots");

    // Bit rot: one byte flipped in the middle of the cloud copy; size and modification time stay.
    let damaged = cloud.join("Policy.pdf");
    let modified = std::fs::metadata(&damaged).and_then(|metadata| metadata.modified()).expect("mtime");
    let mut bytes = std::fs::read(&damaged).expect("read");
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x20;
    std::fs::write(&damaged, &bytes).expect("write");
    std::fs::File::options().write(true).open(&damaged).and_then(|file| file.set_modified(modified)).expect("mtime");

    app.run(|ctx| ctx.check_archive(false));
    let dashboard = app.ctx.dashboard().expect("dashboard");
    let expected = OpenTask::DamagedCopy {
        document_id: policy.id.clone(),
        folder_id: cloud_id.clone(),
        source_folder_id: primary_id,
    };
    assert!(dashboard.tasks.contains(&expected), "{:?}", dashboard.tasks);

    let action =
        crate::features::dashboard::dto::DashboardAction::RepairCopy { task_key: expected.key().value().to_owned() };
    let plan = crate::features::dashboard::plan::plan(&action, &dashboard, &app.catalog()).expect("a plan");
    app.run(|ctx| {
        let services = Arc::clone(ctx.services().expect("services"));
        let context = Arc::clone(ctx);
        crate::features::dashboard::run::execute(
            &ctx.session,
            plan,
            move |call| crate::features::dashboard::run::outcome_of(&services, call),
            move |change| context.forget(change),
        )
    });
    assert_eq!(std::fs::read(&damaged).expect("repaired"), std::fs::read(primary.join("Policy.pdf")).expect("intact"));
    assert_eq!(app.trashed_names(), vec!["Policy.pdf".to_owned()]);
    assert!(!app.ctx.dashboard().expect("dashboard").tasks.contains(&expected), "the finding is gone");

    // A snapshot from before a change brings the old catalog back as a new revision.
    let snapshot = snapshot_before.first().expect("a snapshot after opening").id.clone();
    let before = app.catalog();
    let tags = std::iter::once(tag("Insurance")).collect();
    done(services.library.store.commit(|catalog| Ok(catalog.set_tags(&policy.id, &tags))));
    let changed = app.catalog();
    done(services.sync.restore_catalog_snapshot.run(&snapshot, true));
    let restored = app.catalog();
    assert_eq!(restored.revision(), changed.revision().next());
    assert_eq!(restored.documents(), before.documents());
    let in_cloud = FileCatalogRepository::new(app.trash.clone()).load(&folders.sync_folders()[0].clone());
    assert!(matches!(in_cloud, CatalogLoad::Found { catalog, .. } if catalog.revision() == restored.revision()));
}

#[test]
fn the_emergency_folder_is_exported_as_an_encrypted_zip_and_flags_a_changed_document_after_its_refresh() {
    use std::io::Read as _;

    use dms_application::emergency::{EmergencyEdit, EmergencyOutcome, OverviewTarget};
    use dms_domain::emergency::{EmergencyChanges, EmergencyEntry, EmergencyGroup};

    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    let outside = root.path().join("usb");
    std::fs::create_dir_all(primary.join("Insurance")).expect("folder");
    std::fs::create_dir_all(&outside).expect("folder");
    write_text_pdf(&primary.join("Insurance/Life policy.pdf"), Some("Life insurance policy Example Life"));
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let policy = app.document("Life policy.pdf");
    let organize = &app.services().organize;
    let add = |group, entry| done(organize.edit_emergency_folder.run(&EmergencyEdit::Add { group, entry }));
    add(EmergencyGroup::Insurance, EmergencyEntry::Document(policy.id.clone()));
    add(
        EmergencyGroup::Contacts,
        EmergencyEntry::Contact {
            name: "Example Tax Office".to_owned(),
            role: Some("Tax adviser".to_owned()),
            phone: Some("040 123456".to_owned()),
            email: None,
            note: None,
        },
    );

    let outcome =
        organize.export_emergency_folder.run(&outside.to_string_lossy(), "Emergency folder", "trusted-person");
    let EmergencyOutcome::Saved { location: zip_file, documents: 1, .. } = outcome else {
        panic!("expected the ZIP, got {outcome:?}")
    };
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&zip_file).expect("zip")).expect("archive");
    let mut overview = Vec::new();
    archive
        .by_name_decrypt("Emergency overview.pdf", b"trusted-person")
        .expect("the overview opens with the password")
        .read_to_end(&mut overview)
        .expect("read");
    assert!(overview.starts_with(b"%PDF"));
    let mut pdf = Vec::new();
    archive
        .by_name_decrypt("Life policy.pdf", b"trusted-person")
        .expect("the document")
        .read_to_end(&mut pdf)
        .expect("read");
    assert_eq!(pdf, std::fs::read(primary.join("Insurance/Life policy.pdf")).expect("original"));
    assert!(archive.by_name_decrypt("Life policy.pdf", b"wrong").is_err());

    let saved = organize.create_emergency_overview.run(&OverviewTarget::Library { folder: path("Insurance") });
    assert!(matches!(saved, EmergencyOutcome::Saved { document_id: Some(_), .. }), "{saved:?}");
    done(organize.mark_emergency_folder_refreshed.run());
    let folder = app.catalog().settings().emergency_folder.clone().expect("folder");
    assert_eq!(EmergencyChanges::of(&folder, &app.catalog()).len(), 0);

    // A new version of the policy, as "Replace file" or a rescan records it.
    let hash = dms_domain::document::ContentHash::new(format!("{:064x}", 7)).expect("hash");
    done(app.services().library.store.commit(|catalog| Ok(catalog.update_content(&policy.id, hash.clone(), 1))));
    assert_eq!(EmergencyChanges::of(&folder, &app.catalog()).len(), 1, "the edited policy shows as changed");
}

#[test]
fn an_email_in_the_inbox_becomes_three_linked_documents_with_its_sender_and_date() {
    use dms_domain::settings::InboxSettings;

    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    configure(
        app.services(),
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let inbox = InboxSettings { enabled: true, convert_emails: true, ..InboxSettings::default() };
    done(app.services().organize.configure_inbox.run(&inbox));
    let inbox_folder = primary.join(inbox.path.value());
    std::fs::write(inbox_folder.join("invoice.eml"), SYNTHETIC_MAIL).expect("mail");

    app.run(AppContext::scan);

    let catalog = app.catalog();
    let mail = app.document("2026-11-03 Ihre Rechnung für Oktober.pdf");
    assert_eq!(mail.metadata.correspondent.as_deref(), Some("Stadtwerke Nördlingen"));
    assert_eq!(mail.metadata.document_date, chrono::NaiveDate::from_ymd_opt(2026, 11, 3));
    let attachments: Vec<String> = catalog
        .links_of(&mail.id)
        .iter()
        .map(|link| catalog.documents()[link.other_than(&mail.id)].path.file_name().to_owned())
        .collect();
    assert_eq!(attachments, vec!["Rechnung.pdf".to_owned(), "Geschäftsbedingungen.pdf".to_owned()]);
    assert!(catalog.links_of(&mail.id).iter().all(|link| link.label() == Some("attachment")));
    assert!(catalog.documents().values().all(|document| document.path.file_name() != "invoice.eml"));
    assert_eq!(app.trashed_names(), vec!["invoice.eml".to_owned()]);
    let text = PdfEngine::bind(&PdfEngine::dev_library().expect("PDFium"))
        .expect("PDFium binds")
        .page_texts(&inbox_folder.join("2026-11-03 Ihre Rechnung für Oktober.pdf"))
        .expect("the mail PDF opens");
    assert!(text[0].contains("rechnung@stadtwerke.example") && text[0].contains("anbei Ihre Rechnung."), "{}", text[0]);
}

/// A synthetic mail: HTML body with an inline picture, two PDF attachments and an RFC 2047 encoded subject.
const SYNTHETIC_MAIL: &str = "From: =?UTF-8?Q?Stadtwerke_N=C3=B6rdlingen?= <rechnung@stadtwerke.example>\r
To: Erika Beispiel <erika@example.org>\r
Subject: =?UTF-8?B?SWhyZSBSZWNobnVuZyBmw7xyIE9rdG9iZXI=?=\r
Date: Mon, 3 Nov 2026 23:30:00 +0100\r
MIME-Version: 1.0\r
Content-Type: multipart/mixed; boundary=\"outer\"\r
\r
--outer\r
Content-Type: multipart/related; boundary=\"inner\"\r
\r
--inner\r
Content-Type: text/html; charset=utf-8\r
\r
<html><body><p>Guten Tag,</p><p>anbei Ihre <b>Rechnung</b>.</p><img src=\"cid:logo\"></body></html>\r
--inner\r
Content-Type: image/png\r
Content-ID: <logo>\r
Content-Disposition: inline\r
Content-Transfer-Encoding: base64\r
\r
iVBORw0KGgo=\r
--inner--\r
--outer\r
Content-Type: application/pdf; name=\"Rechnung.pdf\"\r
Content-Disposition: attachment; filename=\"Rechnung.pdf\"\r
Content-Transfer-Encoding: base64\r
\r
JVBERi0xLjQgcmVjaG51bmc=\r
--outer\r
Content-Type: application/pdf\r
Content-Disposition: attachment; filename=\"=?UTF-8?Q?Gesch=C3=A4ftsbedingungen.pdf?=\"\r
Content-Transfer-Encoding: base64\r
\r
JVBERi0xLjQgYWdi\r
--outer--\r
";

#[test]
fn a_scanned_stack_is_split_at_the_proposed_cuts_and_its_parts_wait_for_review() {
    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    for (name, text) in [
        ("a.pdf", "Parking ticket of Sample City for parking without a valid ticket on Main Street"),
        ("b.pdf", "Lease agreement between Sample Housing and the tenant, page one of two pages"),
        ("c.pdf", "Lease agreement between Sample Housing and the tenant, page two of two pages"),
    ] {
        write_text_pdf(&primary.join(name), Some(text));
    }
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    let services = app.services();
    configure(
        services,
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let pages = ["a.pdf", "b.pdf", "c.pdf"].map(|name| app.document(name).id);
    done(services.pdf.merge_documents.run(&pages, "stack", true));
    let stack = app.document("stack.pdf");

    let dms_application::split::ProposeSplitOutcome::Proposed { proposal, model, page_count, source } =
        services.pdf.propose_split.run(&stack.id, &NoProgress)
    else {
        panic!("a proposal")
    };
    assert_eq!((page_count, source), (3, dms_application::split::PageSource::TextLayer));
    assert_eq!(proposal.problem, None);
    let input = dms_application::split::ApplySplitInput {
        document_id: stack.id.clone(),
        parts: proposal.parts,
        model,
        trash_original: false,
        confirmed: true,
    };
    let dms_application::split::ApplySplitOutcome::Split { outcome, parts, without_suggestion } =
        services.pdf.apply_split.run(&input)
    else {
        panic!("a split")
    };

    done(outcome);
    assert_eq!(app.paths().iter().filter(|path| path.starts_with("stack (")).count(), 2, "{:?}", app.paths());
    assert_eq!(without_suggestion, vec![parts[1].clone()]);
    app.ctx.session.changed();
    let reviews: Vec<_> = app
        .ctx
        .dashboard()
        .expect("dashboard")
        .tasks
        .into_iter()
        .filter_map(|task| match task {
            OpenTask::ReviewSuggestion(suggestion) => Some(suggestion),
            _ => None,
        })
        .collect();
    assert_eq!(reviews.len(), 1);
    assert_eq!((&reviews[0].document_id, reviews[0].title.as_deref()), (&parts[0], Some("Parking ticket")));
    assert_eq!(app.document_by_id(&parts[0]).metadata.title, None, "the suggestion is not applied");
    assert!(app.catalog().documents().contains_key(&stack.id), "the original stays");
}

#[test]
fn a_sentence_becomes_library_filters_and_an_unknown_name_is_reported() {
    let root = tempfile::tempdir().expect("temp dir");
    let primary = root.path().join("Documents");
    std::fs::create_dir_all(&primary).expect("folder");
    write_text_pdf(&primary.join("bill.pdf"), Some("Water bill"));
    let app = TestApp::start(&root.path().join("app-data"), root.path().join("trash"));
    let services = app.services();
    configure(
        services,
        &LibraryCommand::CreatePrimary { display_name: "Documents".into(), location: location(&primary) },
    );
    app.run(AppContext::open);
    let bill = app.document("bill.pdf");
    let metadata = dms_domain::document::DocumentMetadata {
        correspondent: Some("Sample Waterworks".into()),
        ..Default::default()
    };
    done(services.library.store.commit(|catalog| catalog.update_metadata(&bill.id, metadata.clone())));

    let dms_application::search::TranslateOutcome::Translated(translation) =
        services.ai.translate_query.run("water bills from 2025 by Unknown Sender GmbH")
    else {
        panic!("a translation")
    };
    let result = crate::features::plain_search::view::result_dto(&translation);

    assert_eq!(
        result.filters,
        vec![
            crate::features::library::dto::params::LibraryFilterDto::Correspondent { name: "Sample Waterworks".into() },
            crate::features::library::dto::params::LibraryFilterDto::Year { year: 2025, and_earlier: false },
        ]
    );
    assert_eq!(result.not_understood, vec!["Unknown Sender GmbH".to_owned()]);
}
