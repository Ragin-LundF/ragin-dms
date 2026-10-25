package dms.ui.library.folders.model

import dms.domain.library.model.RelativePathModel

/** One segment of the breadcrumb; the first one is the library's top level, named "Library". */
data class FolderCrumbModel(
    val path: RelativePathModel,
    val name: String,
)
