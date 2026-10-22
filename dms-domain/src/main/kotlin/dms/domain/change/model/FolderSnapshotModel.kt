package dms.domain.change.model

import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/** All files of one reachable library folder at the time of a scan. */
data class FolderSnapshotModel(
    val folderId: LibraryFolderIdModel,
    val files: Map<RelativePathModel, FileFactsModel>,
)
