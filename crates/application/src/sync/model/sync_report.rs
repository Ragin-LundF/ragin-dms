use std::ops::Add;

use dms_domain::library::{LibraryFolderId, RelativePath};
use indexmap::{IndexMap, IndexSet};

use crate::sync::SyncAction;

/// What a sync did and what it left for the user.
///
/// - `conflicts`: paths whose content differs between folders; nothing was overwritten.
/// - `left_alone`: files the sync deliberately did not touch, per folder: files that exist only in a sync folder,
///   or copies of a moved document that wait for the user's decision.
/// - `failed_paths`: files that could not be copied into the primary folder.
/// - `pending_folders`: folders with operations waiting in the journal.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncReport {
    pub actions: Vec<SyncAction>,
    pub conflicts: Vec<RelativePath>,
    pub left_alone: IndexMap<LibraryFolderId, Vec<RelativePath>>,
    pub failed_paths: Vec<RelativePath>,
    pub pending_folders: IndexSet<LibraryFolderId>,
    pub unreachable_folders: IndexSet<LibraryFolderId>,
}

impl Add for SyncReport {
    type Output = Self;

    fn add(mut self, other: Self) -> Self {
        self.actions.extend(other.actions);
        self.conflicts.extend(other.conflicts);
        for (folder_id, paths) in other.left_alone {
            self.left_alone.entry(folder_id).or_default().extend(paths);
        }
        self.failed_paths.extend(other.failed_paths);
        self.pending_folders.extend(other.pending_folders);
        self.unreachable_folders.extend(other.unreachable_folders);
        self
    }
}
