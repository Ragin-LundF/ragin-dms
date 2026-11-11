use dms_domain::library::RelativePath;
use indexmap::IndexSet;

use crate::port::storage::StorageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoldersOutcome {
    /// Every folder of the primary library folder, also empty ones.
    Read(IndexSet<RelativePath>),
    LibraryNotOpen,
    /// The primary folder could not be read.
    StorageFailed(StorageError),
}
