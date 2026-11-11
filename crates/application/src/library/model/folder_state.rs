use dms_domain::library::LibraryFolder;

/// How one library folder is doing. `reachable` is `None` until the first scan; `pending_operations` counts the
/// journal entries waiting for this folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderState {
    pub folder: LibraryFolder,
    pub reachable: Option<bool>,
    pub pending_operations: usize,
}
