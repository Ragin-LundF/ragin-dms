use crate::port::storage::{FolderEntries, StorageError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderEntriesOutcome {
    /// What lies below the folder in every reachable library folder together.
    Read(FolderEntries),
    LibraryNotOpen,
    /// The primary folder could not be read.
    StorageFailed(StorageError),
}
