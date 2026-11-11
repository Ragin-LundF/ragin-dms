mod file_system_document_storage;
mod file_system_document_storage_move;
mod folder_entries;

use std::path::PathBuf;
use std::sync::Arc;

use dms_domain::library::{FolderRole, LibraryFolder};
use dms_infra_storage::FileSystemDocumentStorage;
use tempfile::TempDir;

use crate::support::{RecordingTrash, folder, folder_with};

/// A primary and a sync folder in a temporary directory, with a recording trash.
pub struct Folders {
    pub temp: TempDir,
    pub primary_root: PathBuf,
    pub sync_root: PathBuf,
    pub primary: LibraryFolder,
    pub sync: LibraryFolder,
    pub trash: Arc<RecordingTrash>,
    pub storage: FileSystemDocumentStorage,
}

impl Folders {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp dir");
        let primary_root = temp.path().join("primary");
        let sync_root = temp.path().join("sync");
        std::fs::create_dir_all(&primary_root).expect("primary");
        std::fs::create_dir_all(&sync_root).expect("sync");
        let trash = Arc::new(RecordingTrash::new(&temp.path().join("trash")));
        Self {
            primary: folder(&primary_root),
            sync: folder_with(&sync_root, "sync", FolderRole::Sync),
            storage: FileSystemDocumentStorage::new(trash.clone()),
            primary_root,
            sync_root,
            temp,
            trash,
        }
    }
}
