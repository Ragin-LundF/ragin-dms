use dms_domain::change::FileFacts;
use dms_domain::document::ContentHash;
use dms_domain::library::{LibraryFolder, RelativePath};
use dms_domain::rebuild::LayoutBox;

use crate::port::ai::PageImage;
use crate::port::pdf::{PageInput, PreparedPage, RebuiltFile};
use crate::port::storage::StorageResult;

/// Rebuilds a scanned PDF as a new PDF with real text where the scan could be read and verified, and the original
/// pixels everywhere else. The result waits in the app's own data, never in a library folder, until the caller
/// places or discards it; what is left over when the app ends is deleted at its next start. The source never
/// changes.
pub trait PdfRebuilder: Send + Sync {
    /// Page `page` of `path` rendered at full resolution and straightened.
    ///
    /// # Errors
    /// Fails when the page cannot be rendered.
    fn prepare_page(&self, folder: &LibraryFolder, path: &RelativePath, page: usize) -> StorageResult<PreparedPage>;

    /// The part of `prepared` around `bounds`, for reading one line again.
    fn crop(&self, prepared: &PreparedPage, bounds: &LayoutBox) -> PageImage;

    /// Writes the rebuilt PDF of `source`, whose content is `source_hash`, from `pages`, in their order.
    ///
    /// # Errors
    /// Fails when the source cannot be read or the result cannot be written.
    fn write(
        &self,
        folder: &LibraryFolder,
        source: &RelativePath,
        source_hash: &ContentHash,
        pages: &[PageInput],
    ) -> StorageResult<RebuiltFile>;

    /// One page of `rebuilt` as a picture, or `None` if it cannot be rendered.
    fn render_page(&self, rebuilt: &RebuiltFile, page: usize, max_side: u32) -> Option<PageImage>;

    /// Copies `rebuilt` to `target` in `folder`, which must not exist yet.
    ///
    /// # Errors
    /// Fails when the target exists or cannot be written.
    fn place(&self, rebuilt: &RebuiltFile, folder: &LibraryFolder, target: &RelativePath) -> StorageResult<FileFacts>;

    /// Deletes `rebuilt`; it was never part of the library.
    fn discard(&self, rebuilt: &RebuiltFile);
}
