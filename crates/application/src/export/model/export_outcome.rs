use dms_domain::DomainError;

use crate::port::storage::StorageError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOutcome {
    /// `count` documents were written to `location`.
    Exported {
        count: usize,
        location: String,
    },
    /// The scope holds no documents; nothing was written.
    Empty,
    /// The target lies inside a library folder; an export never writes into the library.
    InsideLibrary,
    LibraryNotOpen,
    Failed(StorageError),
    /// A blank directory or name (Kotlin threw `IllegalArgumentException`); nothing was written.
    InvalidInput(DomainError),
}
