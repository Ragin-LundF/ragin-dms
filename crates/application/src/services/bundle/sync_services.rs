use std::sync::Arc;

use crate::resolution::{DismissTask, KeepBothVersions, KeepVersion, MergeDuplicates, ResolveMove};
use crate::sync::{MergeAllFolders, RetryFailedOperations, SyncFromPrimary};

/// Synchronization of the library folders and the answers to dashboard tasks.
pub struct SyncServices {
    pub sync_from_primary: Arc<SyncFromPrimary>,
    pub merge_all_folders: Arc<MergeAllFolders>,
    pub retry_failed_operations: Arc<RetryFailedOperations>,
    pub keep_version: Arc<KeepVersion>,
    pub keep_both_versions: Arc<KeepBothVersions>,
    pub resolve_move: Arc<ResolveMove>,
    pub dismiss_task: Arc<DismissTask>,
    pub merge_duplicates: Arc<MergeDuplicates>,
}
