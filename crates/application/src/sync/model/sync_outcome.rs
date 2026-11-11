use crate::port::storage::StorageError;
use crate::sync::SyncReport;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // returned once per sync
pub enum SyncOutcome {
    Completed(SyncReport),
    LibraryNotOpen,
    /// The primary folder could not be scanned or written; nothing was copied.
    PrimaryFailed(StorageError),
}
