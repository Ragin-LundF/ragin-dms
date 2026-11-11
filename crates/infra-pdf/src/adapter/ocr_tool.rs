use dms_application::port::pdf::OcrTool;
use dms_application::port::storage::StorageResult;
use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use super::LibraryLocations;
use super::convert::{self, not_found, storage_error, unreachable};
use crate::OcrMyPdf;

/// "Precise OCR" with the `ocrmypdf` installed on this device, if any.
pub struct OcrMyPdfTool {
    program: OcrMyPdf,
    locations: LibraryLocations,
}

impl OcrMyPdfTool {
    #[must_use]
    pub fn new(program: OcrMyPdf) -> Self {
        Self { program, locations: LibraryLocations }
    }
}

impl OcrTool for OcrMyPdfTool {
    fn available(&self) -> bool {
        self.program.available()
    }

    fn ocr(&self, folder: &LibraryFolder, source: &RelativePath, target: &RelativePath) -> StorageResult<FileFacts> {
        let input = self.locations.existing(folder, source).ok_or_else(not_found)?;
        let output = self.locations.target(folder, target).ok_or_else(unreachable)?;
        self.program.ocr(&input, &output).map(convert::file_facts).map_err(storage_error)
    }
}
