use dms_domain::document::DocumentId;
use dms_domain::library::LibraryFolder;

use crate::port::storage::StorageResult;

/// The extracted text of documents, stored in `.dms-text/` of a library folder for search.
pub trait TextStore: Send + Sync {
    /// # Errors
    /// Fails when the text cannot be written.
    fn write(&self, folder: &LibraryFolder, document_id: &DocumentId, text: &str) -> StorageResult<()>;

    /// The stored text, or `None` if there is none.
    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<String>;
}
