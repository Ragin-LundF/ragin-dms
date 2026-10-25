package dms.ui.library.documents

import dms.ui.library.documents.model.DocumentSortModel
import dms.ui.library.documents.model.SortColumn
import dms.ui.library.model.DocumentRowModel
import java.util.Locale

/**
 * Sorts the document list by the column the user clicked. Rows without a value in that column, such
 * as documents without a date, stay at the end in either direction; equal rows keep their order.
 */
object DocumentOrder {

    /** The sort after a click on [column]: the same column turns around, another one starts ascending. */
    fun next(current: DocumentSortModel?, column: SortColumn): DocumentSortModel {
        return if (current?.column == column) {
            current.copy(descending = !current.descending)
        } else {
            DocumentSortModel(column = column)
        }
    }

    /** [rows] in the order of [sort]; without one, in the order the filter returned them. */
    fun sorted(rows: List<DocumentRowModel>, sort: DocumentSortModel?): List<DocumentRowModel> {
        if (sort == null) {
            return rows
        }
        val (withValue, withoutValue) = rows.partition { hasValue(row = it, column = sort.column) }
        val ascending = comparatorOf(column = sort.column)
        return withValue.sortedWith(comparator = if (sort.descending) ascending.reversed() else ascending) +
            withoutValue
    }

    private fun hasValue(row: DocumentRowModel, column: SortColumn): Boolean {
        val metadata = row.document.metadata
        return when (column) {
            SortColumn.NAME, SortColumn.SIZE -> true
            SortColumn.CATEGORY -> row.categoryName != null
            SortColumn.CORRESPONDENT -> metadata.correspondent != null
            SortColumn.DATE -> metadata.documentDate != null
        }
    }

    private fun comparatorOf(column: SortColumn): Comparator<DocumentRowModel> {
        return when (column) {
            SortColumn.NAME -> compareBy { lowercase(text = it.document.metadata.title ?: it.document.path.fileName) }
            SortColumn.CATEGORY -> compareBy { lowercase(text = it.categoryName.orEmpty()) }
            SortColumn.CORRESPONDENT -> compareBy { lowercase(text = it.document.metadata.correspondent.orEmpty()) }
            SortColumn.DATE -> compareBy { it.document.metadata.documentDate }
            SortColumn.SIZE -> compareBy { it.document.size }
        }
    }

    private fun lowercase(text: String): String {
        return text.lowercase(Locale.ROOT)
    }
}
