use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::port::ai::PageImage;
use crate::port::pdf::{PageChange, PdfMetadata, TextDocument};
use crate::port::storage::StorageResult;

/// Reads and writes PDF files in one library folder. Every write creates a new file at a target that must not
/// exist yet (written to a temporary file and moved into place); the source never changes. Replacing a document's
/// file with the new version is the caller's job.
///
/// Every `# Errors` case is a [`crate::port::storage::StorageError`]: the file is missing or no readable PDF, the
/// target exists, or the file system failed.
pub trait PdfEditor: Send + Sync {
    /// # Errors
    /// See the trait documentation.
    fn page_count(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<usize>;

    /// One page as a small picture for the page editor, or `None` if it cannot be rendered.
    fn render_page(&self, folder: &LibraryFolder, path: &RelativePath, page: usize, max_side: u32)
    -> Option<PageImage>;

    /// A new PDF of the `pages` of `source`, in their order and turned as given; this rotates, deletes, reorders
    /// and extracts.
    ///
    /// # Errors
    /// See the trait documentation.
    fn rewrite(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        pages: &[PageChange],
        target: &RelativePath,
    ) -> StorageResult<FileFacts>;

    /// A new PDF of every page of `sources`, one after the other.
    ///
    /// # Errors
    /// See the trait documentation.
    fn merge(
        &self,
        folder: &LibraryFolder,
        sources: &[RelativePath],
        target: &RelativePath,
    ) -> StorageResult<FileFacts>;

    /// Whether the PDF has a usable text layer on its pages, as a scan without OCR has not.
    ///
    /// # Errors
    /// See the trait documentation.
    fn has_text_layer(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<bool>;

    /// The text layer of every page, in order; empty texts for a scan without one.
    ///
    /// # Errors
    /// See the trait documentation.
    fn page_texts(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<String>>;

    /// For every page, in order, whether it looks blank on a small picture, as a separator sheet or an empty back
    /// side does.
    ///
    /// # Errors
    /// See the trait documentation.
    fn blank_pages(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<Vec<bool>>;

    /// A copy of `source` with `pages` as an invisible text layer, one text per page, so it can be searched.
    ///
    /// # Errors
    /// See the trait documentation.
    fn add_text_layer(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        pages: &[String],
        target: &RelativePath,
    ) -> StorageResult<FileFacts>;

    /// A copy of `source` with `metadata` in its document information and XMP.
    ///
    /// # Errors
    /// See the trait documentation.
    fn write_metadata(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        metadata: &PdfMetadata,
        target: &RelativePath,
    ) -> StorageResult<FileFacts>;

    /// # Errors
    /// See the trait documentation.
    fn read_metadata(&self, folder: &LibraryFolder, path: &RelativePath) -> StorageResult<PdfMetadata>;

    /// A new PDF at `target` with `document` laid out on A4 pages, in fonts that are embedded, so umlauts and the
    /// euro sign stay readable and searchable.
    ///
    /// # Errors
    /// See the trait documentation.
    fn write_text_document(
        &self,
        folder: &LibraryFolder,
        target: &RelativePath,
        document: &TextDocument,
    ) -> StorageResult<FileFacts>;

    /// The PDF [`Self::write_text_document`] writes, as bytes, for a file that goes outside the library, such as
    /// the missing-items page of a tax export.
    ///
    /// # Errors
    /// See the trait documentation.
    fn text_document_bytes(&self, document: &TextDocument) -> StorageResult<Vec<u8>>;
}
