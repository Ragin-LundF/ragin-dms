//! Wiring of the `folder` use cases on `TestLibrary`.

use dms_application::folder::{CreateFolder, MoveFolder, ReadFolderEntries, ReadFolders, TrashFolder};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn create_folder(&self) -> CreateFolder {
        CreateFolder::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
        )
    }

    pub fn move_folder(&self) -> MoveFolder {
        MoveFolder::new(
            self.store.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
        )
    }

    pub fn read_folder_entries(&self) -> ReadFolderEntries {
        ReadFolderEntries::new(self.store.clone(), self.folders.clone())
    }

    pub fn read_folders(&self) -> ReadFolders {
        ReadFolders::new(self.store.clone(), self.folders.clone())
    }

    pub fn trash_folder(&self) -> TrashFolder {
        TrashFolder::new(self.store.clone(), self.replicator.clone(), self.tasks.clone(), self.activity.clone())
    }
}
