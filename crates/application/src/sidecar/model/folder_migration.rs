use dms_domain::library::LibraryFolderId;

use crate::sidecar::FolderMigrationResult;

/// What moving the hidden files of one library folder did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderMigration {
    pub folder_id: LibraryFolderId,
    pub folder_name: String,
    pub result: FolderMigrationResult,
}
