use dms_domain::document::DocumentId;
use dms_domain::library::LibraryFolder;

use crate::port::index::DocumentVectors;
use crate::port::storage::StorageResult;

/// The embedding index of one library folder, one file per document in `.dms-index/`. Named by document id, so
/// moves and renames never touch it; replicated like the text store.
pub trait VectorIndex: Send + Sync {
    /// # Errors
    /// Fails when the vectors cannot be written.
    fn write(&self, folder: &LibraryFolder, vectors: &DocumentVectors) -> StorageResult<()>;

    /// The stored vectors, or `None` if there are none or the file cannot be read.
    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<DocumentVectors>;
}
