package dms.application.sync.model

import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/** One file a sync copied into, or moved within, one library folder. */
data class SyncActionModel(
    val folderId: LibraryFolderIdModel,
    val kind: SyncActionKind,
    val path: RelativePathModel,
)
