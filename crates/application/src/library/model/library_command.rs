use dms_domain::library::{FolderLocation, LibraryFolderId};

/// A change to the library folders of this device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryCommand {
    /// The first configuration: the folder that becomes the primary.
    CreatePrimary {
        display_name: String,
        location: FolderLocation,
    },
    AddSyncFolder {
        display_name: String,
        location: FolderLocation,
    },
    RemoveSyncFolder(LibraryFolderId),
    /// Makes another folder the primary; the old primary becomes a sync folder.
    ChoosePrimary(LibraryFolderId),
}
