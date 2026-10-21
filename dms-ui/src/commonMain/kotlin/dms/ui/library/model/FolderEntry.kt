package dms.ui.library.model

import dms.domain.library.RelativePath

/** One folder of the folder tree, flattened: [depth] is its indentation, [documentCount] counts all below it. */
data class FolderEntry(
    val path: RelativePath,
    val name: String,
    val depth: Int,
    val documentCount: Int,
)
