use dms_domain::health::SnapshotDiff;

use crate::port::storage::StorageError;

/// What restoring a snapshot would change.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotComparison {
    LibraryNotOpen,
    /// The snapshot is gone or cannot be read.
    Failed(StorageError),
    Compared(SnapshotDiff),
}
