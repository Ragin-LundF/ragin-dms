use indexmap::IndexMap;

use crate::change::FileFacts;
use crate::library::{LibraryFolderId, RelativePath};

/// All files of one reachable library folder at the time of a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderSnapshot {
    pub folder_id: LibraryFolderId,
    pub files: IndexMap<RelativePath, FileFacts>,
}
