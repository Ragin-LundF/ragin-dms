use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::catalog::CatalogUpdate;
use dms_domain::document::DocumentId;
use dms_domain::library::RelativePath;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::{ActivityKind, TaskStore};
use crate::sync::{Replication, Replicator};

/// Moves a folder with everything in it to the system trash in every library folder and removes its documents
/// from the catalog, so they can be restored from the trash. Only with the user's confirmation; the library's top
/// level cannot be deleted.
pub struct TrashFolder {
    store: Arc<CatalogStore>,
    replicator: Arc<Replicator>,
    tasks: Arc<dyn TaskStore>,
    activity: Arc<ActivityRecorder>,
}

impl TrashFolder {
    pub fn new(
        store: Arc<CatalogStore>,
        replicator: Arc<Replicator>,
        tasks: Arc<dyn TaskStore>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, replicator, tasks, activity }
    }

    pub fn run(&self, path: &RelativePath, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        if path.is_root() {
            return OperationOutcome::NotFound;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let pending = match self.replicator.trash(&library.folders, path) {
            Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
            Replication::Done { pending_folders, .. } => pending_folders,
        };
        // Taken at commit time, so a document added in the meantime is removed with its folder too.
        let removed: Mutex<Vec<DocumentId>> = Mutex::new(Vec::new());
        let outcome = self
            .store
            .commit(|catalog| {
                let inside: Vec<DocumentId> = catalog
                    .documents()
                    .values()
                    .filter(|document| document.path.is_inside(path))
                    .map(|document| document.id.clone())
                    .collect();
                let update = inside.iter().fold(CatalogUpdate::Applied(catalog.clone()), |update, id| {
                    update.and_then(|current| current.remove_document(id))
                });
                *removed.lock().unwrap_or_else(PoisonError::into_inner) = inside;
                Ok(update)
            })
            .with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            for id in removed.into_inner().unwrap_or_else(PoisonError::into_inner) {
                // Best effort: the documents are gone, a stale suggestion only waits for nothing.
                let _ = self.tasks.remove_suggestion(&id);
            }
            self.activity.record(ActivityKind::FolderTrashed, path.value());
        }
        outcome
    }
}
