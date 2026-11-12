use std::sync::Arc;

use dms_domain::health::SnapshotDiff;

use crate::health::SnapshotComparison;
use crate::library::CatalogStore;
use crate::port::health::{CatalogSnapshotId, CatalogSnapshots};

/// What restoring a catalog snapshot would change in the open catalog; changes nothing.
pub struct CompareCatalogSnapshot {
    store: Arc<CatalogStore>,
    snapshots: Arc<dyn CatalogSnapshots>,
}

impl CompareCatalogSnapshot {
    pub fn new(store: Arc<CatalogStore>, snapshots: Arc<dyn CatalogSnapshots>) -> Self {
        Self { store, snapshots }
    }

    pub fn run(&self, id: &CatalogSnapshotId) -> SnapshotComparison {
        let Some(current) = self.store.catalog() else {
            return SnapshotComparison::LibraryNotOpen;
        };
        match self.snapshots.load(current.library_id(), id) {
            Ok(snapshot) => SnapshotComparison::Compared(SnapshotDiff::between(&current, &snapshot)),
            Err(error) => SnapshotComparison::Failed(error),
        }
    }
}
