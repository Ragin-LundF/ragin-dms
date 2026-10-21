package dms.ui.library.drag.model

import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName

/** Where a dragged document can be dropped, and what the drop does. */
sealed interface DropTarget {

    /** Moves the document into the folder, in every library folder. */
    data class Folder(val path: RelativePath) : DropTarget

    /** Gives the document this category. */
    data class Category(val categoryId: CategoryId) : DropTarget

    /** Removes the document's category. */
    data object Uncategorized : DropTarget

    /** Adds this tag to the document. */
    data class Tag(val tag: TagName) : DropTarget
}
