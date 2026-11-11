use crate::port::state::ActivityEntry;
use crate::port::storage::StorageResult;

/// What the tool did on this device, newest first.
///
/// The Kotlin functions threw when the log could not be read or written; here that is an error value.
pub trait ActivityLog: Send + Sync {
    /// # Errors
    /// Fails when the log cannot be written.
    fn record(&self, entry: &ActivityEntry) -> StorageResult<()>;

    /// # Errors
    /// Fails when the log cannot be read.
    fn recent(&self, limit: usize) -> StorageResult<Vec<ActivityEntry>>;
}
