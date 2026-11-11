use dms_domain::catalog::Catalog;
use dms_domain::document::Document;

use crate::port::ai::DocumentContent;

/// A document read for the AI, or why it could not be read.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)] // one result per read document
pub enum ReadingResult {
    Read {
        document: Document,
        catalog: Catalog,
        content: DocumentContent,
    },
    LibraryNotOpen,
    NotFound,
    /// The file type is never sent to the AI, or the file could not be read. `detail` is technical.
    NotReadable {
        detail: Option<String>,
    },
}
