use std::sync::mpsc::Receiver;

use dms_domain::library::{LibraryFolder, RelativePath};

/// Tells when something changes below `path` in any of `folders`, such as a file dropped into the inbox. Only a
/// hint to scan: the scan stays the source of truth. Folders that cannot be watched are left out quietly.
pub trait FolderWatcher: Send + Sync {
    /// One message per change burst. Dropping the receiver stops watching, as cancelling the Kotlin `Flow` did.
    fn changes(&self, folders: &[LibraryFolder], path: &RelativePath) -> Receiver<()>;
}
