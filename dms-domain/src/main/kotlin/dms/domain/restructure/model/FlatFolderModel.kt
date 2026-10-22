package dms.domain.restructure.model

import dms.domain.library.model.RelativePathModel

/** A folder with many documents and no subfolders, which the restructure assistant could sort. */
data class FlatFolderModel(
    val path: RelativePathModel,
    val fileCount: Int,
)
