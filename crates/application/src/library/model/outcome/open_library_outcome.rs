use dms_domain::catalog::{Catalog, CatalogMergeConflict};
use dms_domain::library::LibraryFolderId;
use indexmap::IndexSet;

use crate::port::storage::StorageError;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // returned once per opening
pub enum OpenLibraryOutcome {
    /// The library is open. `merge_conflicts` are decisions the merge of the catalog copies made;
    /// `unreachable_folders` could not be read; `foreign_folders` hold the catalog of another library and were
    /// ignored; `pending_folders` did not receive the merged catalog yet.
    Opened {
        catalog: Catalog,
        merge_conflicts: Vec<CatalogMergeConflict>,
        unreachable_folders: IndexSet<LibraryFolderId>,
        foreign_folders: IndexSet<LibraryFolderId>,
        pending_folders: IndexSet<LibraryFolderId>,
    },
    /// No library folders are configured (or the settings cannot be read).
    NotConfigured,
    /// The primary folder or its catalog could not be read; nothing was opened.
    PrimaryFailed(StorageError),
}
