use std::sync::Arc;

use crate::health::{CompareCatalogSnapshot, RepairCopy, RestoreCatalogSnapshot, VerifyLibrary};
use crate::port::health::{CatalogSnapshots, HealthStore};
use crate::resolution::{DismissTask, KeepBothVersions, KeepVersion, MergeDuplicates, ResolveMove};
use crate::sync::{MergeAllFolders, RetryFailedOperations, SyncFromPrimary};

/// Synchronization of the library folders, the answers to dashboard tasks, and the archive health: the check, its
/// stored report (`health`), repairs and the catalog snapshots (`snapshots`).
pub struct SyncServices {
    pub sync_from_primary: Arc<SyncFromPrimary>,
    pub merge_all_folders: Arc<MergeAllFolders>,
    pub retry_failed_operations: Arc<RetryFailedOperations>,
    pub keep_version: Arc<KeepVersion>,
    pub keep_both_versions: Arc<KeepBothVersions>,
    pub resolve_move: Arc<ResolveMove>,
    pub dismiss_task: Arc<DismissTask>,
    pub merge_duplicates: Arc<MergeDuplicates>,
    pub verify_library: Arc<VerifyLibrary>,
    pub repair_copy: Arc<RepairCopy>,
    pub compare_catalog_snapshot: Arc<CompareCatalogSnapshot>,
    pub restore_catalog_snapshot: Arc<RestoreCatalogSnapshot>,
    pub health: Arc<dyn HealthStore>,
    pub snapshots: Arc<dyn CatalogSnapshots>,
}
