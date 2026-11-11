use dms_domain::library::LibraryFolder;

use crate::port::export::{ExportBundle, ExportTarget};
use crate::port::storage::StorageResult;

/// Writes copies of library files outside the library, for example for the tax adviser. It only reads the
/// library; every copy is verified by hash.
pub trait ExportWriter: Send + Sync {
    /// `true` if `directory`, an absolute path of this device, is one of `folders` or lies inside one.
    fn is_inside(&self, directory: &str, folders: &[LibraryFolder]) -> bool;

    /// Writes `bundle`, copying from `source`, and returns where it was written.
    ///
    /// # Errors
    /// Fails when a file cannot be read, written or verified.
    fn write(&self, source: &LibraryFolder, bundle: &ExportBundle, target: &ExportTarget) -> StorageResult<String>;

    /// Writes one text file named `name` into `directory` (a free name is chosen) and returns its location.
    ///
    /// # Errors
    /// Fails when the file cannot be written.
    fn write_text(&self, directory: &str, name: &str, text: &str) -> StorageResult<String>;
}
