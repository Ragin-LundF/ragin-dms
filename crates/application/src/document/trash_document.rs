use std::sync::Arc;

use dms_domain::document::DocumentId;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::{ActivityKind, TaskStore};
use crate::sync::{Replication, Replicator};

/// Moves a document to the system trash in every library folder and removes it from the catalog. The only way the
/// tool deletes anything, and only with the user's confirmation.
pub struct TrashDocument {
    store: Arc<CatalogStore>,
    replicator: Arc<Replicator>,
    tasks: Arc<dyn TaskStore>,
    activity: Arc<ActivityRecorder>,
}

impl TrashDocument {
    pub fn new(
        store: Arc<CatalogStore>,
        replicator: Arc<Replicator>,
        tasks: Arc<dyn TaskStore>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, replicator, tasks, activity }
    }

    pub fn run(&self, document_id: &DocumentId, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return OperationOutcome::NotFound;
        };
        let pending = match self.replicator.trash(&library.folders, &document.path) {
            Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
            Replication::Done { pending_folders, .. } => pending_folders,
        };
        let outcome =
            self.store.commit(|catalog| Ok(catalog.remove_document(document_id))).with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            // Best effort: the document is gone, a stale suggestion only waits for a document that no longer exists.
            let _ = self.tasks.remove_suggestion(document_id);
            self.activity.record(ActivityKind::DocumentTrashed, document.path.value());
        }
        outcome
    }
}
