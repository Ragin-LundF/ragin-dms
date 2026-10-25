package dms.ui.library.documents

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.library.documents.model.DocumentGroupModel
import dms.ui.library.documents.model.GroupKey
import dms.ui.library.model.DocumentRowModel
import java.time.YearMonth
import java.time.ZoneId
import java.time.format.TextStyle
import java.util.Locale

/**
 * Groups the shown documents under headers: by month of the document date (or of when a document
 * without one was added), newest first; by top-level category; or by sender. Documents without a
 * category or sender come last. Inside a group the documents keep their order.
 */
object DocumentGroups {

    private const val WITHOUT_CATEGORY = "Without a category"
    private const val UNKNOWN_SENDER = "Unknown sender"

    fun of(rows: List<DocumentRowModel>, key: GroupKey, zone: ZoneId, catalog: CatalogModel): List<DocumentGroupModel> {
        return when (key) {
            GroupKey.MONTH -> byMonth(rows = rows, zone = zone)
            GroupKey.CATEGORY -> byName(rows = rows, missing = WITHOUT_CATEGORY) { document ->
                topCategoryOf(categoryId = document.categoryId, catalog = catalog)
            }

            GroupKey.SENDER -> byName(rows = rows, missing = UNKNOWN_SENDER) { document ->
                document.metadata.correspondent?.trim()
            }
        }
    }

    private fun byMonth(rows: List<DocumentRowModel>, zone: ZoneId): List<DocumentGroupModel> {
        return rows.groupBy { monthOf(document = it.document, zone = zone) }
            .toSortedMap(comparator = Comparator.reverseOrder())
            .map { (month, grouped) -> DocumentGroupModel(title = titleOf(month = month), rows = grouped) }
    }

    /** Groups by [nameOf], ignoring case and showing the first spelling; `null` names go to [missing], last. */
    private fun byName(
        rows: List<DocumentRowModel>,
        missing: String,
        nameOf: (DocumentModel) -> String?,
    ): List<DocumentGroupModel> {
        val (named, unnamed) = rows.partition { nameOf(it.document) != null }
        val groups = named.groupBy { nameOf(it.document).orEmpty().lowercase(Locale.ROOT) }
            .toSortedMap()
            .values
            .map { grouped -> DocumentGroupModel(title = nameOf(grouped.first().document).orEmpty(), rows = grouped) }
        return if (unnamed.isEmpty()) groups else groups + DocumentGroupModel(title = missing, rows = unnamed)
    }

    private fun monthOf(document: DocumentModel, zone: ZoneId): YearMonth {
        val date = document.metadata.documentDate ?: document.addedAt.atZone(zone).toLocalDate()
        return YearMonth.from(date)
    }

    private fun titleOf(month: YearMonth): String {
        return "${month.month.getDisplayName(TextStyle.FULL, Locale.ENGLISH)} ${month.year}"
    }

    private fun topCategoryOf(categoryId: CategoryIdModel?, catalog: CatalogModel): String? {
        val visited = mutableSetOf<CategoryIdModel>()
        var current = categoryId?.let { catalog.categories[it] } ?: return null
        while (visited.add(element = current.id)) {
            current = current.parentId?.let { catalog.categories[it] } ?: break
        }
        return current.name
    }
}
