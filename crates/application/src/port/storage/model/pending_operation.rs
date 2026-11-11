use chrono::{DateTime, Utc};
use dms_domain::library::LibraryFolderId;

use crate::port::storage::FolderOperation;

/// A [`FolderOperation`] that failed for one sync folder and waits to be retried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOperation {
    pub id: String,
    pub folder_id: LibraryFolderId,
    pub operation: FolderOperation,
    pub recorded_at: DateTime<Utc>,
}
