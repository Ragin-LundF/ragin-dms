use std::sync::Arc;

use dms_domain::catalog::Catalog;
use dms_domain::health::SnapshotRetention;

use crate::port::health::CatalogSnapshots;
use crate::port::system::Clock;

/// Keeps a copy of every saved catalog and removes those the retention rule no longer needs.
pub struct CatalogSnapshotKeeper {
    snapshots: Arc<dyn CatalogSnapshots>,
    clock: Arc<dyn Clock>,
}

impl CatalogSnapshotKeeper {
    pub fn new(snapshots: Arc<dyn CatalogSnapshots>, clock: Arc<dyn Clock>) -> Self {
        Self { snapshots, clock }
    }

    /// Best effort: the catalog is already saved in every folder, so a snapshot that cannot be written does not
    /// turn the save into a failure; the next save takes one again.
    pub fn keep(&self, catalog: &Catalog) {
        let now = self.clock.now();
        if self.snapshots.save(catalog, now).is_err() {
            return;
        }
        let Ok(stored) = self.snapshots.list(catalog.library_id()) else { return };
        let taken: Vec<_> = stored.iter().map(|snapshot| snapshot.taken_at).collect();
        for index in SnapshotRetention::to_remove(&taken, now) {
            // A snapshot that cannot be deleted stays; the next save tries again.
            let _ = self.snapshots.remove(catalog.library_id(), &stored[index].id);
        }
    }
}
