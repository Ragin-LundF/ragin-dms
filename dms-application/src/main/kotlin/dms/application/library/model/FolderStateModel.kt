package dms.application.library.model

import dms.domain.library.model.LibraryFolderModel

/**
 * How one library folder is doing. [reachable] is `null` until the first scan; [pendingOperations]
 * counts the journal entries waiting for this folder.
 */
data class FolderStateModel(
    val folder: LibraryFolderModel,
    val reachable: Boolean?,
    val pendingOperations: Int,
)
