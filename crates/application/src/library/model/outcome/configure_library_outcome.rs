use dms_domain::DomainError;
use dms_domain::library::LibraryFolders;

use crate::port::storage::StorageError;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigureLibraryOutcome {
    Configured(LibraryFolders),
    AlreadyConfigured,
    NotConfigured,
    LocationAlreadyUsed,
    UnknownFolder,
    PrimaryCannotBeRemoved,
    /// A rule of the folder model was broken, such as a blank display name (Kotlin threw).
    InvalidInput(DomainError),
    /// The settings could not be read or written (Kotlin threw).
    StorageFailed(StorageError),
}
