package dms.ui.dashboard.model

import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/** A file an open task is about, at [path] in [folderId], with the [label] of the button that opens it. */
data class TaskFileModel(
    val label: String,
    val folderId: LibraryFolderIdModel,
    val path: RelativePathModel,
)
