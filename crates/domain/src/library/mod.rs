//! Library folders, relative paths and the file name rules.

mod file_name_policy;
mod file_name_violation;
mod folder_role;
pub mod model;
mod temporary_names;

pub use file_name_policy::FileNamePolicy;
pub use file_name_violation::FileNameViolation;
pub use folder_role::FolderRole;
pub use model::{FolderLocation, LibraryFolder, LibraryFolderId, LibraryFolders, RelativePath};
pub use temporary_names::TemporaryNames;
