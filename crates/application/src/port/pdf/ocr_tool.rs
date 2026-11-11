use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::port::storage::StorageResult;

/// An OCR program installed on this device, such as `ocrmypdf`; nothing is bundled. It writes a searchable copy
/// of a scan to a target that must not exist yet.
pub trait OcrTool: Send + Sync {
    /// Whether the program is installed and can be run.
    fn available(&self) -> bool;

    /// # Errors
    /// Fails when the program fails or the target cannot be written.
    fn ocr(&self, folder: &LibraryFolder, source: &RelativePath, target: &RelativePath) -> StorageResult<FileFacts>;
}
