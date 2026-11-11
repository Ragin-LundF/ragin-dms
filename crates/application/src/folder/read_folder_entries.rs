use std::sync::Arc;

use dms_domain::library::RelativePath;

use crate::folder::FolderEntriesOutcome;
use crate::library::CatalogStore;
use crate::port::storage::{DocumentStorage, FolderEntries, StorageFailure};

/// Reads what really lies below a folder, in every reachable library folder together: the folders, also empty
/// ones the catalog cannot know, and every file, also files not in the catalog or only in a sync folder. A folder
/// missing in one library folder holds nothing there. Only reads.
pub struct ReadFolderEntries {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
}

impl ReadFolderEntries {
    pub fn new(store: Arc<CatalogStore>, storage: Arc<dyn DocumentStorage>) -> Self {
        Self { store, storage }
    }

    pub fn run(&self, path: &RelativePath) -> FolderEntriesOutcome {
        let Some(library) = self.store.current() else {
            return FolderEntriesOutcome::LibraryNotOpen;
        };
        let primary = self.storage.list_entries(library.folders.primary(), path);
        if let Err(error) = &primary
            && error.reason != StorageFailure::NotFound
        {
            return FolderEntriesOutcome::StorageFailed(error.clone());
        }
        let synced = library.folders.sync_folders().into_iter().map(|folder| self.storage.list_entries(folder, path));
        // An offline sync folder or one without this folder adds nothing; its files come with the next sync.
        let entries: Vec<FolderEntries> = std::iter::once(primary).chain(synced).filter_map(Result::ok).collect();
        FolderEntriesOutcome::Read(FolderEntries {
            folders: entries.iter().flat_map(|entries| entries.folders.iter().cloned()).collect(),
            files: entries.iter().flat_map(|entries| entries.files.iter().cloned()).collect(),
        })
    }
}
