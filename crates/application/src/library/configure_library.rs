use std::sync::Arc;

use dms_domain::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId, LibraryFolders};

use crate::library::{ConfigureLibraryOutcome, LibraryCommand};
use crate::port::state::SettingsRepository;
use crate::port::system::IdGenerator;

/// Sets up the primary folder and adds, removes or promotes sync folders on this device.
pub struct ConfigureLibrary {
    settings: Arc<dyn SettingsRepository>,
    ids: Arc<dyn IdGenerator>,
}

impl ConfigureLibrary {
    pub fn new(settings: Arc<dyn SettingsRepository>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { settings, ids }
    }

    pub fn run(&self, command: &LibraryCommand) -> ConfigureLibraryOutcome {
        let current = match self.settings.load_library_folders() {
            Ok(current) => current,
            Err(error) => return ConfigureLibraryOutcome::StorageFailed(error),
        };
        if let LibraryCommand::CreatePrimary { display_name, location } = command {
            return self.create_primary(current.as_ref(), display_name, location);
        }
        let Some(current) = current else {
            return ConfigureLibraryOutcome::NotConfigured;
        };
        let outcome = match command {
            LibraryCommand::AddSyncFolder { display_name, location } => {
                self.add_sync_folder(&current, display_name, location)
            }
            LibraryCommand::RemoveSyncFolder(folder_id) => remove_sync_folder(&current, folder_id),
            LibraryCommand::ChoosePrimary(folder_id) => choose_primary(&current, folder_id),
            LibraryCommand::CreatePrimary { .. } => ConfigureLibraryOutcome::AlreadyConfigured,
        };
        if let ConfigureLibraryOutcome::Configured(folders) = &outcome
            && let Err(error) = self.settings.save_library_folders(folders)
        {
            return ConfigureLibraryOutcome::StorageFailed(error);
        }
        outcome
    }

    fn create_primary(
        &self,
        current: Option<&LibraryFolders>,
        display_name: &str,
        location: &FolderLocation,
    ) -> ConfigureLibraryOutcome {
        if current.is_some() {
            return ConfigureLibraryOutcome::AlreadyConfigured;
        }
        let folders = self
            .new_folder(display_name, location, FolderRole::Primary)
            .and_then(|primary| LibraryFolders::new(vec![primary]));
        let folders = match folders {
            Ok(folders) => folders,
            Err(error) => return ConfigureLibraryOutcome::InvalidInput(error),
        };
        if let Err(error) = self.settings.save_library_folders(&folders) {
            return ConfigureLibraryOutcome::StorageFailed(error);
        }
        ConfigureLibraryOutcome::Configured(folders)
    }

    fn add_sync_folder(
        &self,
        current: &LibraryFolders,
        display_name: &str,
        location: &FolderLocation,
    ) -> ConfigureLibraryOutcome {
        if current.contains(location) {
            return ConfigureLibraryOutcome::LocationAlreadyUsed;
        }
        configured(self.new_folder(display_name, location, FolderRole::Sync).and_then(|folder| current.add(folder)))
    }

    fn new_folder(
        &self,
        display_name: &str,
        location: &FolderLocation,
        role: FolderRole,
    ) -> Result<LibraryFolder, dms_domain::DomainError> {
        let id = LibraryFolderId::new(self.ids.new_id()).expect("invariant: generated ids are not blank");
        LibraryFolder::new(id, display_name, location.clone(), role)
    }
}

fn remove_sync_folder(current: &LibraryFolders, folder_id: &LibraryFolderId) -> ConfigureLibraryOutcome {
    if current.find(folder_id).is_none() {
        return ConfigureLibraryOutcome::UnknownFolder;
    }
    if &current.primary().id == folder_id {
        return ConfigureLibraryOutcome::PrimaryCannotBeRemoved;
    }
    configured(current.remove(folder_id))
}

fn choose_primary(current: &LibraryFolders, folder_id: &LibraryFolderId) -> ConfigureLibraryOutcome {
    if current.find(folder_id).is_none() {
        return ConfigureLibraryOutcome::UnknownFolder;
    }
    configured(current.with_primary(folder_id))
}

fn configured(folders: Result<LibraryFolders, dms_domain::DomainError>) -> ConfigureLibraryOutcome {
    match folders {
        Ok(folders) => ConfigureLibraryOutcome::Configured(folders),
        Err(error) => ConfigureLibraryOutcome::InvalidInput(error),
    }
}
