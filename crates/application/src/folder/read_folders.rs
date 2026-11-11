use std::sync::Arc;

use crate::folder::FoldersOutcome;
use crate::library::CatalogStore;
use crate::port::storage::DocumentStorage;

/// Reads the folders of the primary library folder, also empty ones the catalog cannot know, for the folder tree
/// and the pickers. Folders only, no files, and only the primary: the sync folders mirror it. Only reads.
pub struct ReadFolders {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
}

impl ReadFolders {
    pub fn new(store: Arc<CatalogStore>, storage: Arc<dyn DocumentStorage>) -> Self {
        Self { store, storage }
    }

    pub fn run(&self) -> FoldersOutcome {
        let Some(library) = self.store.current() else {
            return FoldersOutcome::LibraryNotOpen;
        };
        match self.storage.list_folders(library.folders.primary()) {
            Ok(folders) => FoldersOutcome::Read(folders),
            Err(error) => FoldersOutcome::StorageFailed(error),
        }
    }
}
