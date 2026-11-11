use crate::DomainError;
use crate::error::require;
use crate::library::{FolderLocation, FolderRole, LibraryFolderId};
use crate::text;

/// One folder that holds a complete copy of the library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LibraryFolder {
    pub id: LibraryFolderId,
    pub display_name: String,
    pub location: FolderLocation,
    pub role: FolderRole,
}

impl LibraryFolder {
    /// # Errors
    /// Fails when the display name is blank.
    pub fn new(
        id: LibraryFolderId,
        display_name: impl Into<String>,
        location: FolderLocation,
        role: FolderRole,
    ) -> Result<Self, DomainError> {
        let folder = Self { id, display_name: display_name.into(), location, role };
        folder.validate()?;
        Ok(folder)
    }

    /// # Errors
    /// Fails when the display name is blank.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.display_name), || "Display name must not be blank".to_owned())
    }
}
