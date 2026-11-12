use chrono::{DateTime, Utc};
use dms_domain::catalog::{Catalog, LibraryId};

use crate::port::health::{CatalogSnapshot, CatalogSnapshotId};
use crate::port::storage::StorageResult;

/// Copies of the catalog taken after each save, per library, in the app data directory of this device.
pub trait CatalogSnapshots: Send + Sync {
    /// Stores `catalog` as taken at `taken_at`.
    ///
    /// # Errors
    /// Fails when the copy cannot be written.
    fn save(&self, catalog: &Catalog, taken_at: DateTime<Utc>) -> StorageResult<()>;

    /// The snapshots of `library_id`, newest first.
    ///
    /// # Errors
    /// Fails when the folder cannot be read.
    fn list(&self, library_id: &LibraryId) -> StorageResult<Vec<CatalogSnapshot>>;

    /// # Errors
    /// `NotFound` for an unknown snapshot, `UnsupportedFormat` for one that cannot be read.
    fn load(&self, library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<Catalog>;

    /// Deletes a snapshot; it is the app's own copy, never user data.
    ///
    /// # Errors
    /// Fails when it cannot be deleted.
    fn remove(&self, library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<()>;
}
