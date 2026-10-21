package dms.ui.service

import dms.application.resolution.DismissTask
import dms.application.resolution.KeepBothVersions
import dms.application.resolution.KeepVersion
import dms.application.resolution.MergeDuplicates
import dms.application.resolution.ResolveMove
import dms.application.sync.MergeAllFolders
import dms.application.sync.RetryFailedOperations
import dms.application.sync.SyncFromPrimary

/** Synchronization of the library folders and the answers to dashboard tasks. */
class SyncServices(
    val syncFromPrimary: SyncFromPrimary,
    val mergeAllFolders: MergeAllFolders,
    val retryFailedOperations: RetryFailedOperations,
    val keepVersion: KeepVersion,
    val keepBothVersions: KeepBothVersions,
    val resolveMove: ResolveMove,
    val dismissTask: DismissTask,
    val mergeDuplicates: MergeDuplicates,
)
