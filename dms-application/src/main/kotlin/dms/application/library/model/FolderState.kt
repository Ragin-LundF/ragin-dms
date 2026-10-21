package dms.application.library.model

import dms.domain.library.LibraryFolder

/**
 * How one library folder is doing. [reachable] is `null` until the first scan; [pendingOperations]
 * counts the journal entries waiting for this folder.
 */
data class FolderState(
    val folder: LibraryFolder,
    val reachable: Boolean?,
    val pendingOperations: Int,
)
