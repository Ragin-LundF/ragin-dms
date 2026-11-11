use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::library::LibraryFolder;

use crate::port::storage::StorageResult;

/// The stored preview (JPEG) of a document in one library folder, kept per content hash, so a changed file never
/// shows its old picture. Derived data: a missing preview is not an error, it is rendered again.
pub trait PreviewStore: Send + Sync {
    /// The JPEG stored for `content_hash`, or `None` if there is none.
    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId, content_hash: &ContentHash) -> Option<Vec<u8>>;

    /// Stores `jpeg` for `content_hash` and removes the previews of other content hashes of the document.
    ///
    /// # Errors
    /// Fails when the preview cannot be written.
    fn write(
        &self,
        folder: &LibraryFolder,
        document_id: &DocumentId,
        content_hash: &ContentHash,
        jpeg: &[u8],
    ) -> StorageResult<()>;
}
