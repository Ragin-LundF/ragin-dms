use dms_domain::library::{LibraryFolderId, RelativePath};

use crate::sync::SyncActionKind;

/// One file a sync copied into, or moved within, one library folder.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyncAction {
    pub folder_id: LibraryFolderId,
    pub kind: SyncActionKind,
    pub path: RelativePath,
}
