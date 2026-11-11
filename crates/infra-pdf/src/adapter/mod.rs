//! The port adapters on the engine: they resolve library paths to files, call the engine and map its types and
//! errors to the domain and port types. Every adapter only reads its sources; every result is a new file.

mod content;
mod convert;
mod file_previewer;
mod locations;
mod ocr_tool;
mod pdfium_editor;
mod pdfium_rebuilder;

use std::path::PathBuf;

pub use content::{FileContentReader, SettingsSource};
pub use file_previewer::FilePreviewer;
pub use locations::LibraryLocations;
pub use ocr_tool::OcrMyPdfTool;
pub use pdfium_editor::PdfiumEditor;
pub use pdfium_rebuilder::PdfiumRebuilder;

use crate::{OcrMyPdf, PdfEngine};

/// The PDF adapters for one device: the editor, the installed OCR program, if any, the rebuild, whose results wait
/// in `rebuild_directory` in the app's own data, the thumbnails and the document reader for the AI and search.
pub struct PdfAdapters {
    pub editor: PdfiumEditor,
    pub ocr: OcrMyPdfTool,
    pub rebuilder: PdfiumRebuilder,
    pub previewer: FilePreviewer,
    pub reader: FileContentReader,
}

impl PdfAdapters {
    /// Make it once when the app starts: that is when leftover rebuilds of an earlier run are deleted.
    #[must_use]
    pub fn new(engine: PdfEngine, rebuild_directory: PathBuf, settings: SettingsSource) -> Self {
        Self {
            editor: PdfiumEditor::new(engine),
            ocr: OcrMyPdfTool::new(OcrMyPdf::default()),
            rebuilder: PdfiumRebuilder::new(engine, rebuild_directory),
            previewer: FilePreviewer::new(engine),
            reader: FileContentReader::new(engine, settings),
        }
    }
}

#[cfg(test)]
mod tests;
