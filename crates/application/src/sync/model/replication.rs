use dms_domain::library::LibraryFolderId;
use indexmap::IndexSet;

use crate::port::storage::StorageError;

/// The result of applying one change to the primary folder and every sync folder.
#[derive(Debug, Clone, PartialEq)]
pub enum Replication<T> {
    /// The primary was changed; `pending_folders` failed and were recorded in the journal.
    Done { value: T, pending_folders: IndexSet<LibraryFolderId> },
    /// The primary could not be changed, so no folder was touched.
    PrimaryFailed(StorageError),
}
