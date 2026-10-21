package dms.application.sync.model

import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/** One file a sync copied into, or moved within, one library folder. */
data class SyncAction(
    val folderId: LibraryFolderId,
    val kind: SyncActionKind,
    val path: RelativePath,
)
