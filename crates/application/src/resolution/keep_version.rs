use std::sync::Arc;

use dms_domain::library::{LibraryFolderId, RelativePath};

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation, StorageError, StorageFailure};
use crate::sync::Replicator;

/// Makes the version of a path in one folder the version of every folder: for a content conflict, or to accept or
/// revert a file modified outside the tool. The replaced versions go to the system trash, and the catalog records
/// the kept content. Requires the user's confirmation.
pub struct KeepVersion {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
}

impl KeepVersion {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, activity }
    }

    pub fn run(&self, path: &RelativePath, source_folder_id: &LibraryFolderId, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(source) = library.folders.find(source_folder_id) else {
            return OperationOutcome::NotFound;
        };
        let primary = library.folders.primary();
        let kept = if source == primary {
            self.storage.read_facts(primary, path)
        } else {
            self.storage.copy_file(source, primary, path, true)
        };
        let facts = match kept {
            Ok(facts) => facts,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let Some(content_hash) = facts.content_hash else {
            return OperationOutcome::StorageFailed(StorageError::new(StorageFailure::IoError));
        };
        let targets: Vec<_> = library.folders.sync_folders().into_iter().filter(|other| *other != source).collect();
        let pending = self.replicator.distribute(
            &library.folders,
            &targets,
            &FolderOperation::CopyFromPrimary { path: path.clone(), replace_existing: true },
        );
        self.activity.record(ActivityKind::ConflictResolved, path.value());
        let document_id = match library.catalog.find_by_path(path) {
            Some(document) if document.content_hash != content_hash => document.id.clone(),
            _ => return OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: pending },
        };
        self.store
            .commit(|catalog| Ok(catalog.update_content(&document_id, content_hash.clone(), facts.size)))
            .with_pending_folders(&pending)
    }
}
