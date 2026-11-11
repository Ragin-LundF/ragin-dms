//! The PDF engine of ragin-dms on PDFium (`pdfium-render`) and `lopdf`, with a path-level API: sources
//! are only read, and every result is a new file written to a temporary file next to its target and
//! moved into place, never over an existing file. The functions are named after the port methods
//! (`PdfEditor`, `PdfRebuilder`, `OcrTool`, `DocumentPreviewer` and the PDF reading of the AI adapter),
//! so the adapters only map types. Everything is synchronous; callers run it on a blocking thread.
//!
//! [`adapter`] implements the application's PDF-side ports on the engine.

pub mod adapter;
mod document;
mod engine;
mod error;
mod files;
mod fonts;
mod images;
mod ocr;
mod rebuild;
#[cfg(test)]
mod test_support;

pub use document::{DocumentContent, PageChange, PdfDate, PdfMetadata};
pub use engine::PdfEngine;
pub use error::PdfError;
pub use files::{FileFacts, TEMPORARY_PREFIX};
pub use images::{LAYOUT_SIDE, PageImage, VISION_MAX_SIDE, layout_png, vision_jpeg};
pub use ocr::{CommandRunner, OcrMyPdf, ProcessRunner};
pub use rebuild::{
    FontFamily, KeepReason, LayoutBox, LayoutGraphic, LayoutLine, PageInput, PageLayout, PreparedPage, RebuildRegion,
    Rebuilder, RebuiltFile, RebuiltPage, RegionKind,
};
