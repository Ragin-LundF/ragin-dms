//! Creating, moving, reading and trashing folders.

mod create_folder;
pub mod model;
mod move_folder;
mod read_folder_entries;
mod read_folders;
mod trash_folder;

pub use create_folder::CreateFolder;
pub use model::{FolderEntriesOutcome, FoldersOutcome};
pub use move_folder::MoveFolder;
pub use read_folder_entries::ReadFolderEntries;
pub use read_folders::ReadFolders;
pub use trash_folder::TrashFolder;
