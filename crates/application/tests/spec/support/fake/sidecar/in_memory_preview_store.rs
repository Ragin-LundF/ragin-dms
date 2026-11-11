use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dms_application::port::sidecar::PreviewStore;
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::document::{ContentHash, DocumentId};
use dms_domain::library::{LibraryFolder, LibraryFolderId};

use crate::support::Shared;

type Previews = HashMap<LibraryFolderId, HashMap<DocumentId, (ContentHash, Vec<u8>)>>;

/// The stored previews of every library folder, in memory: one per document, for one content hash, as writing a
/// preview removes those of other hashes. Folders in `unreachable` fail.
pub struct InMemoryPreviewStore {
    unreachable: Arc<Shared<HashSet<LibraryFolderId>>>,
    pub entries: Shared<Previews>,
}

impl InMemoryPreviewStore {
    pub fn new(unreachable: Arc<Shared<HashSet<LibraryFolderId>>>) -> Self {
        Self { unreachable, entries: Shared::default() }
    }

    /// The content hash and bytes stored in `folder` for `document_id`.
    pub fn stored(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<(ContentHash, Vec<u8>)> {
        self.entries.lock().get(&folder.id).and_then(|entries| entries.get(document_id)).cloned()
    }
}

impl PreviewStore for InMemoryPreviewStore {
    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId, content_hash: &ContentHash) -> Option<Vec<u8>> {
        self.stored(folder, document_id).filter(|(hash, _)| hash == content_hash).map(|(_, jpeg)| jpeg)
    }

    fn write(
        &self,
        folder: &LibraryFolder,
        document_id: &DocumentId,
        content_hash: &ContentHash,
        jpeg: &[u8],
    ) -> StorageResult<()> {
        if self.unreachable.lock().contains(&folder.id) {
            return Err(StorageError::new(StorageFailure::FolderUnreachable));
        }
        self.entries.update(|entries| {
            entries
                .entry(folder.id.clone())
                .or_default()
                .insert(document_id.clone(), (content_hash.clone(), jpeg.to_vec()))
        });
        Ok(())
    }
}
