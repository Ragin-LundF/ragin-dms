use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use dms_application::port::sidecar::{SidecarFile, SidecarLayout, SidecarMove};
use dms_application::port::storage::{StorageError, StorageFailure, StorageResult};
use dms_domain::library::{LibraryFolder, LibraryFolderId};
use indexmap::IndexMap;

use crate::support::Shared;

/// The hidden per-document files of every library folder, in memory, with their content. Folders in
/// `unreachable` fail.
pub struct FakeSidecarLayout {
    unreachable: Arc<Shared<HashSet<LibraryFolderId>>>,
    pub folders: Shared<HashMap<LibraryFolderId, SidecarFolder>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SidecarFolder {
    pub flat: IndexMap<SidecarFile, String>,
    pub subfolders: IndexMap<SidecarFile, String>,
    /// Flat files that cannot be moved, for example because they are locked.
    pub locked: HashSet<SidecarFile>,
    pub marked: bool,
    /// How often the flat files were listed.
    pub listings: usize,
}

impl FakeSidecarLayout {
    pub fn new(unreachable: Arc<Shared<HashSet<LibraryFolderId>>>) -> Self {
        Self { unreachable, folders: Shared::default() }
    }

    pub fn folder(&self, folder: &LibraryFolder) -> SidecarFolder {
        self.folders.lock().get(&folder.id).cloned().unwrap_or_default()
    }

    pub fn change(&self, folder: &LibraryFolder, change: impl FnOnce(&mut SidecarFolder)) {
        self.folders.update(|folders| change(folders.entry(folder.id.clone()).or_default()));
    }

    fn reachable(&self, folder: &LibraryFolder) -> StorageResult<()> {
        if self.unreachable.lock().contains(&folder.id) {
            return Err(StorageError::new(StorageFailure::FolderUnreachable));
        }
        Ok(())
    }
}

impl SidecarLayout for FakeSidecarLayout {
    fn is_current(&self, folder: &LibraryFolder) -> bool {
        self.reachable(folder).is_ok() && self.folder(folder).marked
    }

    fn flat_files(&self, folder: &LibraryFolder) -> StorageResult<Vec<SidecarFile>> {
        self.reachable(folder)?;
        let mut files = Vec::new();
        self.change(folder, |state| {
            state.listings += 1;
            files = state.flat.keys().cloned().collect();
        });
        Ok(files)
    }

    fn move_into_subfolder(&self, folder: &LibraryFolder, file: &SidecarFile) -> StorageResult<SidecarMove> {
        self.reachable(folder)?;
        let mut result = Err(StorageError::new(StorageFailure::NotFound));
        self.change(folder, |state| {
            if state.locked.contains(file) {
                result = Err(StorageError::new(StorageFailure::IoError));
            } else if let Some(existing) = state.subfolders.get(file) {
                result = Ok(SidecarMove::KeptExisting { identical: state.flat.get(file) == Some(existing) });
            } else if let Some(content) = state.flat.shift_remove(file) {
                state.subfolders.insert(file.clone(), content);
                result = Ok(SidecarMove::Moved);
            }
        });
        result
    }

    fn mark_current(&self, folder: &LibraryFolder) -> StorageResult<()> {
        self.reachable(folder)?;
        self.change(folder, |state| state.marked = true);
        Ok(())
    }
}
