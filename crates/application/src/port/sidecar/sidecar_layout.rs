use dms_domain::library::LibraryFolder;

use crate::port::sidecar::{SidecarFile, SidecarMove};
use crate::port::storage::StorageResult;

/// The layout of the hidden per-document folders (`.dms-text/`, `.dms-index/`) of one library folder: version 1.x
/// kept every file flat in them, version 2 keeps each in a subfolder named after the first two characters of the
/// document id.
pub trait SidecarLayout: Send + Sync {
    /// Whether the folder was moved to the current layout completely (its marker is there).
    fn is_current(&self, folder: &LibraryFolder) -> bool;

    /// The files still in the flat layout.
    ///
    /// # Errors
    /// Fails when the folder is unreachable or cannot be listed.
    fn flat_files(&self, folder: &LibraryFolder) -> StorageResult<Vec<SidecarFile>>;

    /// Moves one flat file into its subfolder. Never replaces a file already there: then both stay as they are.
    ///
    /// # Errors
    /// Fails when the file cannot be moved.
    fn move_into_subfolder(&self, folder: &LibraryFolder, file: &SidecarFile) -> StorageResult<SidecarMove>;

    /// Records that the folder uses the current layout.
    ///
    /// # Errors
    /// Fails when the marker cannot be written.
    fn mark_current(&self, folder: &LibraryFolder) -> StorageResult<()>;
}
