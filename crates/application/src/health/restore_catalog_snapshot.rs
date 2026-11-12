use std::sync::Arc;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::health::{CatalogSnapshotId, CatalogSnapshots};
use crate::port::state::ActivityKind;

/// Makes a catalog snapshot the catalog again, as a new revision saved to every folder, so every device takes it
/// over. Files are never touched; a scan afterwards shows where they and the restored catalog differ. Requires the
/// user's confirmation.
pub struct RestoreCatalogSnapshot {
    store: Arc<CatalogStore>,
    snapshots: Arc<dyn CatalogSnapshots>,
    activity: Arc<ActivityRecorder>,
}

impl RestoreCatalogSnapshot {
    pub fn new(
        store: Arc<CatalogStore>,
        snapshots: Arc<dyn CatalogSnapshots>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, snapshots, activity }
    }

    pub fn run(&self, id: &CatalogSnapshotId, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(current) = self.store.catalog() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let snapshot = match self.snapshots.load(current.library_id(), id) {
            Ok(snapshot) => snapshot,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let outcome = self.store.commit(|catalog| catalog.restored_from(&snapshot));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, &format!("catalog of revision {}", snapshot.revision()));
        }
        outcome
    }
}
