package dms.application.sync.model

import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/**
 * What a sync did and what it left for the user.
 *
 * - [conflicts]: paths whose content differs between folders; nothing was overwritten.
 * - [leftAlone]: files the sync deliberately did not touch, per folder: files that exist only in a
 *   sync folder, or copies of a moved document that wait for the user's decision.
 * - [failedPaths]: files that could not be copied into the primary folder.
 * - [pendingFolders]: folders with operations waiting in the journal.
 */
data class SyncReport(
    val actions: List<SyncAction> = emptyList(),
    val conflicts: List<RelativePath> = emptyList(),
    val leftAlone: Map<LibraryFolderId, List<RelativePath>> = emptyMap(),
    val failedPaths: List<RelativePath> = emptyList(),
    val pendingFolders: Set<LibraryFolderId> = emptySet(),
    val unreachableFolders: Set<LibraryFolderId> = emptySet(),
) {

    operator fun plus(other: SyncReport): SyncReport {
        return SyncReport(
            actions = actions + other.actions,
            conflicts = conflicts + other.conflicts,
            leftAlone = (leftAlone.keys + other.leftAlone.keys).associateWith { folderId ->
                leftAlone[folderId].orEmpty() + other.leftAlone[folderId].orEmpty()
            },
            failedPaths = failedPaths + other.failedPaths,
            pendingFolders = pendingFolders + other.pendingFolders,
            unreachableFolders = unreachableFolders + other.unreachableFolders,
        )
    }
}
