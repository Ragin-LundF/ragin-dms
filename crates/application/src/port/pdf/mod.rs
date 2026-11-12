//! Writing PDF files: page tools, text layers, metadata, OCR and rebuilding scans.

pub mod model;
mod ocr_tool;
mod pdf_editor;
mod pdf_rebuilder;

pub use model::{PageChange, PageInput, PdfMetadata, PreparedPage, RebuiltFile, TextBlock, TextDocument};
pub use ocr_tool::OcrTool;
pub use pdf_editor::PdfEditor;
pub use pdf_rebuilder::PdfRebuilder;
