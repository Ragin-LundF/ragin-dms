use dms_application::port::state::{Preferences, SettingsRepository};
use dms_application::port::storage::StorageResult;
use dms_domain::library::LibraryFolders;

use crate::support::Shared;

#[derive(Default)]
pub struct InMemorySettings {
    pub folders: Shared<Option<LibraryFolders>>,
    pub preferences: Shared<Preferences>,
}

impl InMemorySettings {
    pub fn new(folders: Option<LibraryFolders>) -> Self {
        Self { folders: Shared::new(folders), preferences: Shared::default() }
    }
}

impl SettingsRepository for InMemorySettings {
    fn load_library_folders(&self) -> StorageResult<Option<LibraryFolders>> {
        Ok(self.folders.get())
    }

    fn save_library_folders(&self, folders: &LibraryFolders) -> StorageResult<()> {
        self.folders.set(Some(folders.clone()));
        Ok(())
    }

    fn load_preferences(&self) -> StorageResult<Preferences> {
        Ok(self.preferences.get())
    }

    fn save_preferences(&self, preferences: &Preferences) -> StorageResult<()> {
        self.preferences.set(preferences.clone());
        Ok(())
    }
}
