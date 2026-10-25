package dms.application.port.storage.model

import dms.domain.library.model.RelativePathModel

/**
 * What lies on disk below a folder of one library folder, the tool's own files left out: every
 * [folders] (also empty ones) and every [files], as paths from the library folder's root.
 */
data class FolderEntriesModel(
    val folders: Set<RelativePathModel> = emptySet(),
    val files: Set<RelativePathModel> = emptySet(),
)
