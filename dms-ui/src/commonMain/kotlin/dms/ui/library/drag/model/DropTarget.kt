package dms.ui.library.drag.model

import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel

/** Where a dragged document or category can be dropped, and what the drop does. */
sealed interface DropTarget {

    /** Moves the document into the folder, in every library folder. */
    data class Folder(val path: RelativePathModel) : DropTarget

    /** Gives the document this category. */
    data class Category(val categoryId: CategoryIdModel) : DropTarget

    /** Removes the document's category. */
    data object Uncategorized : DropTarget

    /** Adds this tag to the document. */
    data class Tag(val tag: TagNameModel) : DropTarget

    /** Makes a dragged category a top-level category. */
    data object TopLevelCategory : DropTarget
}
