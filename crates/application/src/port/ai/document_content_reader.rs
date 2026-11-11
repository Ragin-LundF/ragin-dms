use dms_domain::document::Document;
use dms_domain::library::LibraryFolder;

use crate::port::ai::DocumentContent;

/// Turns a document into text or page images for the classifier. Never modifies the file.
pub trait DocumentContentReader: Send + Sync {
    fn read(&self, folder: &LibraryFolder, document: &Document) -> DocumentContent;

    /// The whole text of a PDF's text layer or of an Office file, for search; `None` for images, scans and files
    /// without text. Never renders pages, so it stays cheap.
    fn text(&self, folder: &LibraryFolder, document: &Document) -> Option<String>;
}
