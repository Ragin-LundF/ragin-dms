use dms_domain::catalog::Catalog;
use dms_domain::change::FolderSnapshot;
use dms_domain::library::LibraryFolder;

use crate::port::storage::{ScanListener, StorageResult};

/// Lists every file of one library folder. Files the `catalog` does not know are always hashed; known files only
/// when they changed since the last scan (see [`dms_domain::change::FileFacts`]). The [`ScanListener`] follows the
/// progress and can stop the scan between two files.
pub trait LibraryScanner: Send + Sync {
    /// Pass [`crate::port::storage::NoScanListener`] where Kotlin used the default `ScanListener.NONE`.
    ///
    /// # Errors
    /// Fails when the folder cannot be read, or with
    /// [`crate::port::storage::StorageFailure::Cancelled`] when the listener asked to stop; a partial snapshot is
    /// never returned, because it would report the unread files as missing.
    fn scan(
        &self,
        folder: &LibraryFolder,
        catalog: &Catalog,
        listener: &dyn ScanListener,
    ) -> StorageResult<FolderSnapshot>;
}
