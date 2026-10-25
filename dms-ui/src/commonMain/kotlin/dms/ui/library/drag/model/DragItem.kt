package dms.ui.library.drag.model

import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryModel

/** What the user drags inside the library. */
sealed interface DragItem {

    /** A document, to file it in a folder, under a category or with a tag. */
    data class DraggedDocument(val document: DocumentModel) : DragItem

    /** A category of the sidebar, to move it into another category or to the top level. */
    data class DraggedCategory(val category: CategoryModel) : DragItem

    /** A folder of the folder view, to move it with everything in it into another folder. */
    data class DraggedFolder(val path: RelativePathModel) : DragItem
}
