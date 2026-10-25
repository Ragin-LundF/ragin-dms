package dms.ui.library.documents.model

/** The order the user chose for the document list: by [column], ascending unless [descending]. */
data class DocumentSortModel(
    val column: SortColumn,
    val descending: Boolean = false,
)
