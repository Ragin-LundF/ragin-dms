use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogRejection};
use dms_domain::library::{FileNameViolation, LibraryFolderId};
use indexmap::IndexSet;

use crate::port::storage::StorageError;

/// The outcome shared by the use cases that change files or the catalog.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // returned once per user action
pub enum OperationOutcome {
    /// The change is in the primary folder and the catalog. `pending_folders` are sync folders that could not be
    /// updated yet; their operations wait in the journal.
    Done {
        catalog: Catalog,
        pending_folders: IndexSet<LibraryFolderId>,
    },
    /// No library is configured or opened yet.
    LibraryNotOpen,
    NotFound,
    /// The folders no longer look the way the task describes; scan again and decide anew.
    ScanOutdated,
    /// The file changed after the work on it began; nothing was changed.
    SourceChanged,
    /// A destructive operation was requested without the user's confirmation.
    NotConfirmed,
    InvalidName(FileNameViolation),
    /// Another entry in the target folder already has this name (ignoring case).
    NameTaken,
    Rejected(CatalogRejection),
    /// The primary folder could not be changed; nothing was changed anywhere.
    StorageFailed(StorageError),
    /// The request broke a rule the domain checks, such as a blank category name. Kotlin threw an
    /// `IllegalArgumentException` here; nothing was changed.
    InvalidInput(DomainError),
}

impl OperationOutcome {
    /// Adds `folders` to the pending folders of a [`Self::Done`] outcome; every other outcome stays as it is.
    #[must_use]
    pub fn with_pending_folders(self, folders: &IndexSet<LibraryFolderId>) -> Self {
        match self {
            Self::Done { catalog, mut pending_folders } => {
                pending_folders.extend(folders.iter().cloned());
                Self::Done { catalog, pending_folders }
            }
            other => other,
        }
    }
}
