use std::sync::Arc;

use dms_domain::change::FileFacts;
use dms_domain::document::Document;
use dms_domain::library::{LibraryFolder, RelativePath, TemporaryNames};

use crate::common::{LibrarySnapshot, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::ai::DocumentContentReader;
use crate::port::storage::{DocumentStorage, FolderOperation, StorageResult, TextStore};
use crate::port::system::IdGenerator;
use crate::sync::Replicator;

/// A changed PDF is always a new version of its document: written next to it under a temporary name, put in place
/// in the primary with the old version going to the system trash, copied over the old version in every sync folder
/// (which trashes it there too), and recorded in the catalog. The text the search uses is read anew from the new
/// file.
pub struct PdfVersions {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    reader: Arc<dyn DocumentContentReader>,
    texts: Arc<dyn TextStore>,
    ids: Arc<dyn IdGenerator>,
}

impl PdfVersions {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        reader: Arc<dyn DocumentContentReader>,
        texts: Arc<dyn TextStore>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, storage, replicator, reader, texts, ids }
    }

    /// Replaces `document`'s file with what `write` creates at the temporary path it is given.
    pub fn replace(
        &self,
        library: &LibrarySnapshot,
        document: &Document,
        write: impl FnOnce(&LibraryFolder, &RelativePath) -> StorageResult<FileFacts>,
    ) -> OperationOutcome {
        let primary = library.folders.primary();
        let temporary = match TemporaryNames::sibling_of(&document.path, &self.ids.new_id()) {
            Ok(temporary) => temporary,
            Err(error) => return OperationOutcome::InvalidInput(error),
        };
        if let Err(error) = write(primary, &temporary) {
            return OperationOutcome::StorageFailed(error);
        }
        let facts = match self.storage.replace_with(primary, &document.path, &temporary) {
            Ok(facts) => facts,
            Err(error) => {
                let _ = self.storage.move_to_trash(primary, &temporary);
                return OperationOutcome::StorageFailed(error);
            }
        };
        let pending = self.replicator.distribute(
            &library.folders,
            &library.folders.sync_folders(),
            &FolderOperation::CopyFromPrimary { path: document.path.clone(), replace_existing: true },
        );
        let content_hash = facts.content_hash.expect("invariant: a written file is always hashed");
        let outcome = self
            .store
            .commit(|catalog| Ok(catalog.update_content(&document.id, content_hash.clone(), facts.size)))
            .with_pending_folders(&pending);
        if let OperationOutcome::Done { catalog, .. } = &outcome {
            self.refresh_text(library, catalog.documents().get(&document.id).unwrap_or(document));
        }
        outcome
    }

    /// Derived data: when the new file has a text layer, it is what search finds from now on.
    fn refresh_text(&self, library: &LibrarySnapshot, document: &Document) {
        let Some(text) = self.reader.text(library.folders.primary(), document) else {
            return;
        };
        if dms_domain::text::is_blank(&text) {
            return;
        }
        self.replicator.replicate(
            &library.folders,
            |primary| self.texts.write(primary, &document.id, &text),
            &FolderOperation::CopyText(document.id.clone()),
        );
    }
}
