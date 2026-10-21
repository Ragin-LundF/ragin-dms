package dms.ui.dashboard.model

import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/** A file an open task is about, at [path] in [folderId], with the [label] of the button that opens it. */
data class TaskFile(
    val label: String,
    val folderId: LibraryFolderId,
    val path: RelativePath,
)
