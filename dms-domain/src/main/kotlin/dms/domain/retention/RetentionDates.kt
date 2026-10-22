package dms.domain.retention

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.taxonomy.model.CategoryIdModel
import java.time.LocalDate
import java.time.ZoneOffset

/**
 * When a document's retention ends. Nothing has a retention period unless the user set one: a
 * keep-until date on the document wins; otherwise the period of its category, or of the closest
 * parent category that has one, counts from the end of the document date's year.
 */
object RetentionDates {

    /** The date the retention ends, or `null` if the user set none or already marked it done. */
    fun dueDate(document: DocumentModel, catalog: CatalogModel): LocalDate? {
        val retention = document.retention
        if (retention?.settledAt != null) {
            return null
        }
        retention?.keepUntil?.let { return it }
        val years = categoryYears(categoryId = document.categoryId, catalog = catalog) ?: return null
        val year = (document.metadata.documentDate ?: document.addedAt.atZone(ZoneOffset.UTC).toLocalDate()).year
        return LocalDate.of(year, LAST_MONTH, LAST_DAY).plusYears(years.toLong())
    }

    private fun categoryYears(categoryId: CategoryIdModel?, catalog: CatalogModel): Int? {
        val visited = mutableSetOf<CategoryIdModel>()
        var current = categoryId?.let { catalog.categories[it] }
        while (current != null && visited.add(element = current.id)) {
            current.retentionYears?.let { return it }
            current = current.parentId?.let { catalog.categories[it] }
        }
        return null
    }

    private const val LAST_MONTH = 12
    private const val LAST_DAY = 31
}
