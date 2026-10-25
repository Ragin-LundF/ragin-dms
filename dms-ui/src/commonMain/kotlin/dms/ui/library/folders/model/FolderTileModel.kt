package dms.ui.library.folders.model

import dms.domain.library.model.RelativePathModel

/** A subfolder of the open folder: [documentCount] counts every document below it, [folderCount] its own subfolders. */
data class FolderTileModel(
    val path: RelativePathModel,
    val name: String,
    val documentCount: Int,
    val folderCount: Int,
)
