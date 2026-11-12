use chrono::{DateTime, Utc};
use dms_application::port::health::{CatalogSnapshot, CatalogSnapshotId, CatalogSnapshots};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::catalog::{Catalog, LibraryId};

use crate::support::Shared;

/// Snapshots in memory, oldest first.
#[derive(Default)]
pub struct InMemorySnapshots {
    pub stored: Shared<Vec<(CatalogSnapshot, Catalog)>>,
}

impl InMemorySnapshots {
    pub fn ids(&self) -> Vec<CatalogSnapshotId> {
        self.stored.lock().iter().map(|(snapshot, _)| snapshot.id.clone()).collect()
    }
}

impl CatalogSnapshots for InMemorySnapshots {
    fn save(&self, catalog: &Catalog, taken_at: DateTime<Utc>) -> StorageResult<()> {
        let id = CatalogSnapshotId::new(format!("{}-r{}", taken_at.format("%Y%m%dT%H%M%SZ"), catalog.revision()))
            .expect("valid id");
        let snapshot = CatalogSnapshot { id, taken_at, revision: catalog.revision() };
        self.stored.update(|stored| stored.push((snapshot, catalog.clone())));
        Ok(())
    }

    fn list(&self, library_id: &LibraryId) -> StorageResult<Vec<CatalogSnapshot>> {
        let stored = self.stored.lock();
        let mine = stored.iter().filter(|(_, catalog)| catalog.library_id() == library_id);
        Ok(mine.rev().map(|(snapshot, _)| snapshot.clone()).collect())
    }

    fn load(&self, library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<Catalog> {
        let stored = self.stored.lock();
        stored
            .iter()
            .find(|(snapshot, catalog)| &snapshot.id == id && catalog.library_id() == library_id)
            .map(|(_, catalog)| catalog.clone())
            .ok_or_else(|| StorageError::new(StorageFailure::NotFound))
    }

    fn remove(&self, _library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<()> {
        self.stored.update(|stored| stored.retain(|(snapshot, _)| &snapshot.id != id));
        Ok(())
    }
}
