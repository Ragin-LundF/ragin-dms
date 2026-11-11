use std::sync::Arc;

use dms_application::common::{ActivityRecorder, DocumentDates, OperationOutcome, SiblingNames};
use dms_application::document::{AddDocumentsOutcome, AddedFile};
use dms_application::library::{CatalogStore, OpenLibraryOutcome};
use dms_application::port::storage::ExternalFile;
use dms_application::search::ContentMatcher;
use dms_application::sync::Replicator;
use dms_domain::catalog::Catalog;
use dms_domain::document::Document;
use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, LibraryFolders, RelativePath};

use crate::support::fake::ai::{
    FakeAssistant, FakeClassifier, FakeContentReader, FakeEmbeddings, FakeLibraryAssistant,
};
use crate::support::fake::export::FakeExportWriter;
use crate::support::fake::index::InMemoryVectorIndex;
use crate::support::fake::state::{InMemoryActivityLog, InMemorySettings, InMemoryTaskStore};
use crate::support::fake::storage::{FakeFolders, InMemoryJournal, InMemoryPlanStore};
use crate::support::fake::system::{FixedClock, SequentialIds};
use crate::support::test_data::{document_id, instant, path};

/// A library with a primary and two sync folders, all in memory. Only synthetic names and contents; never real
/// documents.
///
/// It holds the fakes and the shared, stateful parts (`store`, `matcher`); every use case is built on demand by a
/// method in `support::wiring::<feature>`, wired with these, so all use cases of one test share one library.
pub struct TestLibrary {
    pub primary: LibraryFolder,
    pub sync_a: LibraryFolder,
    pub sync_b: LibraryFolder,

    pub folders: Arc<FakeFolders>,
    pub settings: Arc<InMemorySettings>,
    pub journal: Arc<InMemoryJournal>,
    pub tasks: Arc<InMemoryTaskStore>,
    pub activity_log: Arc<InMemoryActivityLog>,
    pub clock: Arc<FixedClock>,
    pub ids: Arc<SequentialIds>,
    pub classifier: Arc<FakeClassifier>,
    pub reader: Arc<FakeContentReader>,
    pub assistant: Arc<FakeAssistant>,
    pub plans: Arc<InMemoryPlanStore>,
    pub index: Arc<InMemoryVectorIndex>,
    pub embeddings: Arc<FakeEmbeddings>,
    pub library_assistant: Arc<FakeLibraryAssistant>,
    pub exports: Arc<FakeExportWriter>,

    pub replicator: Arc<Replicator>,
    pub store: Arc<CatalogStore>,
    pub activity: Arc<ActivityRecorder>,
    pub names: Arc<SiblingNames>,
    pub dates: Arc<DocumentDates>,
    pub matcher: Arc<ContentMatcher>,
}

impl Default for TestLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl TestLibrary {
    pub fn new() -> Self {
        let primary = folder("primary", FolderRole::Primary);
        let sync_a = folder("sync-a", FolderRole::Sync);
        let sync_b = folder("sync-b", FolderRole::Sync);
        let all = LibraryFolders::new(vec![primary.clone(), sync_a.clone(), sync_b.clone()]).expect("valid folders");
        let folders = Arc::new(FakeFolders::default());
        let settings = Arc::new(InMemorySettings::new(Some(all)));
        let journal = Arc::new(InMemoryJournal::default());
        let clock = Arc::new(FixedClock::default());
        let ids = Arc::new(SequentialIds::default());
        let plans = Arc::new(InMemoryPlanStore::new(folders.unreachable.clone()));
        let index = Arc::new(InMemoryVectorIndex::new(folders.unreachable.clone()));
        let reader = Arc::new(FakeContentReader::default());
        let activity_log = Arc::new(InMemoryActivityLog::default());
        let replicator = Arc::new(Replicator::new(
            folders.clone(),
            folders.clone(),
            folders.clone(),
            plans.clone(),
            index.clone(),
            journal.clone(),
            clock.clone(),
            ids.clone(),
        ));
        let store = Arc::new(CatalogStore::new(settings.clone(), folders.clone(), replicator.clone()));
        Self {
            activity: Arc::new(ActivityRecorder::new(activity_log.clone(), clock.clone())),
            names: Arc::new(SiblingNames::new(folders.clone())),
            dates: Arc::new(DocumentDates::new(folders.clone())),
            matcher: Arc::new(ContentMatcher::new(reader.clone(), folders.clone(), clock.clone())),
            primary,
            sync_a,
            sync_b,
            settings,
            journal,
            tasks: Arc::new(InMemoryTaskStore::default()),
            activity_log,
            classifier: Arc::new(FakeClassifier::default()),
            reader,
            assistant: Arc::new(FakeAssistant::default()),
            plans,
            index,
            embeddings: Arc::new(FakeEmbeddings::default()),
            library_assistant: Arc::new(FakeLibraryAssistant::default()),
            exports: Arc::new(FakeExportWriter::default()),
            replicator,
            store,
            clock,
            ids,
            folders,
        }
    }

    pub fn all_folders(&self) -> Vec<LibraryFolder> {
        vec![self.primary.clone(), self.sync_a.clone(), self.sync_b.clone()]
    }

    /// The open catalog; fails the test while no library is open.
    #[track_caller]
    pub fn catalog(&self) -> Catalog {
        self.store.catalog().expect("a library is open")
    }

    /// Opens the library and returns the opened catalog.
    #[track_caller]
    pub fn open(&self) -> Catalog {
        match self.open_library().run() {
            OpenLibraryOutcome::Opened { catalog, .. } => catalog,
            other => panic!("expected Opened, got {other:?}"),
        }
    }

    /// Adds a file with `content` at `raw_path` to every folder through the tool and returns its document.
    #[track_caller]
    pub fn add(&self, raw_path: &str, content: &str) -> Document {
        let target = path(raw_path);
        let source = self.folders.external(&format!("/outside/{}", target.file_name()), content);
        let parent = target.parent().unwrap_or_else(RelativePath::root);
        let added = self.added(&[source], &parent).pop().expect("one file added");
        assert!(matches!(added.outcome, OperationOutcome::Done { .. }), "expected Done, got {:?}", added.outcome);
        self.catalog().find_by_path(&target).cloned().expect("the added document is catalogued")
    }

    /// The files `add_documents` added; fails the test while no library is open.
    #[track_caller]
    pub fn added(&self, files: &[ExternalFile], target_folder: &RelativePath) -> Vec<AddedFile> {
        match self.add_documents().run(files, target_folder) {
            AddDocumentsOutcome::Added(files) => files,
            AddDocumentsOutcome::LibraryNotOpen => panic!("expected Added, got LibraryNotOpen"),
        }
    }

    /// A document for catalogs built by hand; `content` decides its hash and size.
    pub fn document(id: &str, raw_path: &str, content: &str) -> Document {
        let facts = FakeFolders::facts_of(content);
        Document::new(
            document_id(id),
            path(raw_path),
            FakeFolders::hash_of(content),
            facts.size,
            instant("2024-01-01T10:00:00Z"),
        )
        .expect("valid document")
    }
}

fn folder(id: &str, role: FolderRole) -> LibraryFolder {
    LibraryFolder::new(
        LibraryFolderId::new(id).expect("valid id"),
        id,
        FolderLocation::new(format!("/library/{id}")).expect("valid location"),
        role,
    )
    .expect("valid folder")
}
