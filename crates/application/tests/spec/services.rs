//! `AppServices` wires every use case on one set of adapters: what one bundle changes, the others see.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use dms_application::document::AddDocumentsOutcome;
use dms_application::library::OpenLibraryOutcome;
use dms_application::port::storage::FolderWatcher;
use dms_application::search::SearchOutcome;
use dms_application::services::{AppPorts, AppServices};
use dms_domain::library::{LibraryFolder, RelativePath};
use indexmap::IndexSet;

use crate::support::TestLibrary;
use crate::support::fake::ai::{FakeAiServer, FakePreviewer};
use crate::support::fake::pdf::{FakeLayoutReader, FakeOcr, FakePdfEditor, FakePdfRebuilder};

struct NoWatcher;

impl FolderWatcher for NoWatcher {
    fn changes(&self, _folders: &[LibraryFolder], _path: &RelativePath) -> Receiver<()> {
        channel().1
    }
}

fn services_of(library: &TestLibrary) -> AppServices {
    AppServices::new(AppPorts {
        storage: library.folders.clone(),
        scanner: library.folders.clone(),
        catalogs: library.folders.clone(),
        texts: library.folders.clone(),
        plans: library.plans.clone(),
        index: library.index.clone(),
        previews: library.previews.clone(),
        sidecar_layout: library.sidecar_layout.clone(),
        journal: library.journal.clone(),
        watcher: Arc::new(NoWatcher),
        settings: library.settings.clone(),
        tasks: library.tasks.clone(),
        activity: library.activity_log.clone(),
        exports: library.exports.clone(),
        clock: library.clock.clone(),
        ids: library.ids.clone(),
        reader: library.reader.clone(),
        classifier: library.classifier.clone(),
        ai_server: Arc::new(FakeAiServer::default()),
        previewer: Arc::new(FakePreviewer::default()),
        assistant: library.assistant.clone(),
        library_assistant: library.library_assistant.clone(),
        embeddings: library.embeddings.clone(),
        layout_reader: Arc::new(FakeLayoutReader::default()),
        pdf_editor: Arc::new(FakePdfEditor::new(library.folders.clone())),
        ocr: Arc::new(FakeOcr::new(library.folders.clone())),
        rebuilder: Arc::new(FakePdfRebuilder::new(library.folders.clone())),
    })
}

#[test]
fn every_bundle_works_on_the_one_open_library() {
    let library = TestLibrary::new();
    let services = services_of(&library);
    let changes = services.library.store.subscribe();

    assert!(matches!(services.library.open_library.run(), OpenLibraryOutcome::Opened { .. }));
    let file = library.folders.external("/outside/bill.pdf", "electricity bill");
    let added = services.documents.add_documents.run(&[file], &RelativePath::root());
    assert!(matches!(added, AddDocumentsOutcome::Added(_)), "{added:?}");
    library.reader.set_text("bill.pdf", "Electricity bill");

    let found = services.library.search_contents.run("electricity");

    let catalog = services.library.store.catalog().expect("a library is open");
    let bill = catalog.documents().values().next().expect("the added document");
    assert_eq!(found, SearchOutcome::Found(IndexSet::from([bill.id.clone()])));
    assert_eq!(changes.try_iter().last(), Some(Some(catalog)));
}
