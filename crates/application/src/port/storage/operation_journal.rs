use crate::port::storage::{PendingOperation, StorageResult};

/// Operations that could not be applied to a sync folder, kept locally until they are retried.
///
/// The Kotlin functions returned nothing and threw when the journal file could not be read or written; here that
/// is an error value.
pub trait OperationJournal: Send + Sync {
    /// # Errors
    /// Fails when the journal cannot be written.
    fn record(&self, operation: &PendingOperation) -> StorageResult<()>;

    /// # Errors
    /// Fails when the journal cannot be read.
    fn pending(&self) -> StorageResult<Vec<PendingOperation>>;

    /// # Errors
    /// Fails when the journal cannot be written.
    fn remove(&self, id: &str) -> StorageResult<()>;
}
