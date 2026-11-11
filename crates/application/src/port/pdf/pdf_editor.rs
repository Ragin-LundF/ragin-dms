use dms_domain::change::FileFacts;
use dms_domain::library::{LibraryFolder, RelativePath};

use crate::port::ai::PageImage;
use crate::port::pdf::{PageChange, PdfMetadata};
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
}
