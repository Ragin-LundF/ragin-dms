//! Use cases other features build on; each feature wires its own use cases in `wiring/<feature>.rs`.

use std::sync::Arc;

use dms_application::common::DocumentFiling;
use dms_application::document::{AddDocuments, AdoptNewFile, MoveDocument, TrashDocument};
use dms_application::library::OpenLibrary;
use dms_application::search::{FindDocuments, SearchContents};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn open_library(&self) -> OpenLibrary {
        OpenLibrary::new(
            self.settings.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.store.clone(),
            self.tasks.clone(),
            self.ids.clone(),
        )
    }

    pub fn add_documents(&self) -> AddDocuments {
        AddDocuments::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn adopt_new_file(&self) -> AdoptNewFile {
        AdoptNewFile::new(
            self.store.clone(),
            self.folders.clone(),
            self.dates.clone(),
            self.replicator.clone(),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn move_document(&self) -> MoveDocument {
        MoveDocument::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
        )
    }

    pub fn trash_document(&self) -> TrashDocument {
        TrashDocument::new(self.store.clone(), self.replicator.clone(), self.tasks.clone(), self.activity.clone())
    }

    pub fn filing(&self) -> DocumentFiling {
        DocumentFiling::new(self.store.clone(), self.names.clone(), Arc::new(self.move_document()))
    }

    pub fn search_contents(&self) -> SearchContents {
        SearchContents::new(self.store.clone(), self.matcher.clone())
    }

    pub fn find_documents(&self) -> FindDocuments {
        FindDocuments::new(self.store.clone(), self.matcher.clone())
    }
}
