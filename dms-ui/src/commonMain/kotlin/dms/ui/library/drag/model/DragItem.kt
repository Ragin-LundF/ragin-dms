package dms.ui.library.drag.model

import dms.domain.document.Document
import dms.domain.taxonomy.Category

/** What the user drags inside the library. */
sealed interface DragItem {

    /** A document, to file it in a folder, under a category or with a tag. */
    data class DraggedDocument(val document: Document) : DragItem

    /** A category of the sidebar, to move it into another category or to the top level. */
    data class DraggedCategory(val category: Category) : DragItem
}
