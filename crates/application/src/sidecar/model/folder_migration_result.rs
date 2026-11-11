use crate::port::sidecar::SidecarFile;
use crate::port::storage::StorageError;
use crate::sidecar::KeptSidecarFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderMigrationResult {
    /// The folder already uses the subfolders; nothing was looked at.
    AlreadyCurrent,
    /// The folder could not be listed, for example while it is offline; it is moved when it is next reachable.
    Failed(StorageError),
    Moved {
        moved: usize,
        /// Flat files left in place because their subfolder already holds a file for the document.
        kept: Vec<KeptSidecarFile>,
        /// Flat files that could not be moved; they stay where they are and the next run tries again.
        failed: Vec<(SidecarFile, StorageError)>,
        /// Whether the folder was finished and marked, so the next run skips it. Not after a cancel or a failure.
        marked: bool,
    },
}
