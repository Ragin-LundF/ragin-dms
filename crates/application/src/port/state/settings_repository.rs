use dms_domain::library::LibraryFolders;

use crate::port::state::Preferences;
use crate::port::storage::StorageResult;

/// Settings of this device. Folder locations differ per device and are never stored in the catalog.
///
/// The Kotlin functions threw when the settings could not be read or written; here that is an error value.
pub trait SettingsRepository: Send + Sync {
    /// The configured library folders, or `None` before the first configuration.
    ///
    /// # Errors
    /// Fails when the settings cannot be read.
    fn load_library_folders(&self) -> StorageResult<Option<LibraryFolders>>;

    /// # Errors
    /// Fails when the settings cannot be written.
    fn save_library_folders(&self, folders: &LibraryFolders) -> StorageResult<()>;

    /// The preferences of this device, with defaults for everything never set.
    ///
    /// # Errors
    /// Fails when the settings cannot be read.
    fn load_preferences(&self) -> StorageResult<Preferences>;

    /// # Errors
    /// Fails when the settings cannot be written.
    fn save_preferences(&self, preferences: &Preferences) -> StorageResult<()>;
}
