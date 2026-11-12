use dms_domain::library::RelativePath;

use crate::port::health::HealthReport;
use crate::port::storage::StorageError;

/// One document of the check: `done` of `total` documents, `current` the one being read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyProgress {
    pub done: usize,
    pub total: usize,
    pub current: RelativePath,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerifyOutcome {
    LibraryNotOpen,
    /// Every document was checked; the report is saved.
    Finished(HealthReport),
    /// Stopped between two documents; the report is saved and the next run continues where this one stopped.
    Cancelled(HealthReport),
    /// The report of this device could not be read or written.
    StorageFailed(StorageError),
}
