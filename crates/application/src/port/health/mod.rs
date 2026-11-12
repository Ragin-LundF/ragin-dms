//! What this device keeps about the health of the archive: the results of the last check and the catalog
//! snapshots, both in the app data directory.

mod catalog_snapshots;
mod health_store;
pub mod model;

pub use catalog_snapshots::CatalogSnapshots;
pub use health_store::HealthStore;
pub use model::{CatalogSnapshot, CatalogSnapshotId, FolderCheck, HealthReport};
