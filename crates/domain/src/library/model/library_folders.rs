use std::collections::HashSet;

use crate::DomainError;
use crate::error::require;
use crate::library::{FolderLocation, FolderRole, LibraryFolder, LibraryFolderId};

/// The folders of one library on this device: exactly one primary and any number of sync folders.
///
/// Functions return an error for requests that break these rules; callers check user input with
/// [`Self::contains`] and [`Self::find`] first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryFolders {
    folders: Vec<LibraryFolder>,
}

impl LibraryFolders {
    /// # Errors
    /// Fails without exactly one primary, with a repeated id or a location used twice.
    pub fn new(folders: Vec<LibraryFolder>) -> Result<Self, DomainError> {
        for folder in &folders {
            folder.validate()?;
        }
        require(folders.iter().filter(|folder| folder.role == FolderRole::Primary).count() == 1, || {
            "A library needs exactly one primary folder".to_owned()
        })?;
        require(folders.iter().map(|folder| &folder.id).collect::<HashSet<_>>().len() == folders.len(), || {
            "Library folder ids must be unique".to_owned()
        })?;
        require(folders.iter().map(|folder| &folder.location).collect::<HashSet<_>>().len() == folders.len(), || {
            "A location can only be used by one library folder".to_owned()
        })?;
        Ok(Self { folders })
    }

    #[must_use]
    pub fn folders(&self) -> &[LibraryFolder] {
        &self.folders
    }

    /// # Panics
    /// Never: the constructor guarantees exactly one primary.
    #[must_use]
    pub fn primary(&self) -> &LibraryFolder {
        self.folders.iter().find(|folder| folder.role == FolderRole::Primary).expect("invariant: one primary")
    }

    #[must_use]
    pub fn sync_folders(&self) -> Vec<&LibraryFolder> {
        self.folders.iter().filter(|folder| folder.role == FolderRole::Sync).collect()
    }

    #[must_use]
    pub fn find(&self, id: &LibraryFolderId) -> Option<&LibraryFolder> {
        self.folders.iter().find(|folder| &folder.id == id)
    }

    #[must_use]
    pub fn contains(&self, location: &FolderLocation) -> bool {
        self.folders.iter().any(|folder| &folder.location == location)
    }

    /// Adds a sync folder.
    ///
    /// # Errors
    /// Fails for a primary folder or when the result breaks the rules of [`Self::new`].
    pub fn add(&self, folder: LibraryFolder) -> Result<Self, DomainError> {
        require(folder.role == FolderRole::Sync, || "Only sync folders can be added; use withPrimary".to_owned())?;
        let mut folders = self.folders.clone();
        folders.push(folder);
        Self::new(folders)
    }

    /// Removes a sync folder. The primary cannot be removed; choose another primary first.
    ///
    /// # Errors
    /// Fails for the primary folder.
    pub fn remove(&self, id: &LibraryFolderId) -> Result<Self, DomainError> {
        require(&self.primary().id != id, || "The primary folder cannot be removed".to_owned())?;
        Self::new(self.folders.iter().filter(|folder| &folder.id != id).cloned().collect())
    }

    /// Makes `id` the primary folder; the previous primary becomes a sync folder.
    ///
    /// # Errors
    /// Fails for an unknown folder.
    pub fn with_primary(&self, id: &LibraryFolderId) -> Result<Self, DomainError> {
        require(self.find(id).is_some(), || format!("Unknown library folder: {id}"))?;
        let folders = self
            .folders
            .iter()
            .map(|folder| {
                let role = if &folder.id == id { FolderRole::Primary } else { FolderRole::Sync };
                LibraryFolder { role, ..folder.clone() }
            })
            .collect();
        Self::new(folders)
    }
}
