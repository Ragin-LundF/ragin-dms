package dms.application.sync.model

import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * What a sync did and what it left for the user.
 *
 * - [conflicts]: paths whose content differs between folders; nothing was overwritten.
 * - [leftAlone]: files the sync deliberately did not touch, per folder: files that exist only in a
 *   sync folder, or copies of a moved document that wait for the user's decision.
 * - [failedPaths]: files that could not be copied into the primary folder.
 * - [pendingFolders]: folders with operations waiting in the journal.
 */
data class SyncReportModel(
    val actions: List<SyncActionModel> = emptyList(),
    val conflicts: List<RelativePathModel> = emptyList(),
    val leftAlone: Map<LibraryFolderIdModel, List<RelativePathModel>> = emptyMap(),
    val failedPaths: List<RelativePathModel> = emptyList(),
    val pendingFolders: Set<LibraryFolderIdModel> = emptySet(),
    val unreachableFolders: Set<LibraryFolderIdModel> = emptySet(),
) {

    operator fun plus(other: SyncReportModel): SyncReportModel {
        return SyncReportModel(
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
