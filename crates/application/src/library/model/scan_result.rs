use dms_domain::change::DetectedChange;
use dms_domain::library::LibraryFolderId;
use indexmap::{IndexMap, IndexSet};

use crate::port::storage::StorageError;

/// Everything one scan of the library folders found.
#[derive(Debug, Clone, PartialEq)]
pub struct ScanResult {
    pub changes: Vec<DetectedChange>,
    pub scanned_folders: IndexSet<LibraryFolderId>,
    pub failed_folders: IndexMap<LibraryFolderId, StorageError>,
}
