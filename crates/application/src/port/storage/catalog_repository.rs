use dms_domain::catalog::{Catalog, CatalogRevision};
use dms_domain::library::LibraryFolder;

use crate::port::storage::{CatalogLoad, StorageResult};

/// Reads and writes the catalog file of one library folder.
pub trait CatalogRepository: Send + Sync {
    fn load(&self, folder: &LibraryFolder) -> CatalogLoad;

    /// Writes `catalog`, unless the file on disk holds a higher revision than `based_on`, the newest revision the
    /// caller has seen: then nothing is written and the result is
    /// [`crate::port::storage::StorageFailure::CatalogChanged`], so a change another device saved in the meantime
    /// is never overwritten.
    ///
    /// # Errors
    /// Fails as described, or when the file cannot be written.
    fn save(&self, folder: &LibraryFolder, catalog: &Catalog, based_on: CatalogRevision) -> StorageResult<()>;

    /// Moves the conflict copies of the catalog file to the trash, after they were merged and saved.
    ///
    /// # Errors
    /// Fails when a copy cannot be moved.
    fn discard_conflict_copies(&self, folder: &LibraryFolder) -> StorageResult<()>;
}
