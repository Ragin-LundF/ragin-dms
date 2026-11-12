//! The archive health state of this device: `health.yaml` and the catalog snapshots in `snapshots/`.

mod file_catalog_snapshots;
mod file_health_store;

pub use file_catalog_snapshots::FileCatalogSnapshots;
pub use file_health_store::FileHealthStore;
