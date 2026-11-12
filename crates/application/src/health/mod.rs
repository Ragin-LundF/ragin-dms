//! Archive health: checking every copy of every document, repairing a damaged copy from an intact one, and the
//! catalog snapshots taken after each save.

mod catalog_snapshot_keeper;
mod compare_catalog_snapshot;
pub mod model;
mod repair_copy;
mod restore_catalog_snapshot;
mod verify_library;

pub use catalog_snapshot_keeper::CatalogSnapshotKeeper;
pub use compare_catalog_snapshot::CompareCatalogSnapshot;
pub use model::{SnapshotComparison, VerifyOutcome, VerifyProgress};
pub use repair_copy::RepairCopy;
pub use restore_catalog_snapshot::RestoreCatalogSnapshot;
pub use verify_library::VerifyLibrary;
