use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::port::index::{DocumentVectors, VectorIndex};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolder, LibraryFolderId};

use crate::support::Shared;

/// `.dms-index/` of every library folder, in memory; folders in `unreachable` fail.
pub struct InMemoryVectorIndex {
    unreachable: Arc<Shared<HashSet<LibraryFolderId>>>,
    pub entries: Shared<HashMap<LibraryFolderId, HashMap<DocumentId, DocumentVectors>>>,
    writes: AtomicUsize,
}

impl InMemoryVectorIndex {
    pub fn new(unreachable: Arc<Shared<HashSet<LibraryFolderId>>>) -> Self {
        Self { unreachable, entries: Shared::default(), writes: AtomicUsize::new(0) }
    }

    pub fn writes(&self) -> usize {
        self.writes.load(Ordering::SeqCst)
    }
}

impl VectorIndex for InMemoryVectorIndex {
    fn write(&self, folder: &LibraryFolder, vectors: &DocumentVectors) -> StorageResult<()> {
        if self.unreachable.lock().contains(&folder.id) {
            return Err(StorageError::new(StorageFailure::FolderUnreachable));
        }
        self.writes.fetch_add(1, Ordering::SeqCst);
        self.entries.update(|entries| {
            entries.entry(folder.id.clone()).or_default().insert(vectors.document_id.clone(), vectors.clone())
        });
        Ok(())
    }

    fn read(&self, folder: &LibraryFolder, document_id: &DocumentId) -> Option<DocumentVectors> {
        self.entries.lock().get(&folder.id).and_then(|entries| entries.get(document_id)).cloned()
    }
}
