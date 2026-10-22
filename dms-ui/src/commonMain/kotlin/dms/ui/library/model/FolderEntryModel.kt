package dms.ui.library.model

import dms.domain.library.model.RelativePathModel

/**
 * One folder of the folder tree, flattened: [depth] is its indentation, [documentCount] counts all
 * below it. Subfolders are listed only while the folder is [expanded].
 */
data class FolderEntryModel(
    val path: RelativePathModel,
    val name: String,
    val depth: Int,
    val documentCount: Int,
    val hasChildren: Boolean = false,
    val expanded: Boolean = false,
)
