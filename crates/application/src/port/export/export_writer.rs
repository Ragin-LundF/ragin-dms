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

    /// Writes `content` as a file named `name` into `directory` (a free name is chosen) and returns its location.
    ///
    /// # Errors
    /// Fails when the file cannot be written.
    fn write_file(&self, directory: &str, name: &str, content: &[u8]) -> StorageResult<String>;

    /// Writes `bundle` and the `extra` files (name, content) into the ZIP file of `target`, every entry encrypted
    /// with AES-256 under `password`, and returns where it was written. The password is neither kept nor logged.
    /// The bundle's index is left out.
    ///
    /// # Errors
    /// Fails when a file cannot be read, written or verified.
    fn write_encrypted_zip(
        &self,
        source: &LibraryFolder,
        bundle: &ExportBundle,
        extra: &[(String, Vec<u8>)],
        target: &ExportTarget,
        password: &str,
    ) -> StorageResult<String>;
}
