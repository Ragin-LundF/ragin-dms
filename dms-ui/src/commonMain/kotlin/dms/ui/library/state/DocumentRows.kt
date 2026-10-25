package dms.ui.library.state

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldRole
import dms.domain.field.FieldValue
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.library.LibraryFilter
import dms.ui.library.LibraryTrees
import dms.ui.library.documents.model.CategoryTileModel
import dms.ui.library.model.DocumentRowModel
import dms.ui.library.sidebar.model.SidebarCountsModel
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.util.Locale

/** The rows of the documents with their amount and due date, the category tiles, and the sidebar's counts. */
object DocumentRows {

    private const val TILE_PAGES = 3

    fun rowOf(document: DocumentModel, catalog: CatalogModel): DocumentRowModel {
        return DocumentRowModel(
            document = document,
            categoryName = document.categoryId?.let { catalog.categories[it]?.name },
            amount = amountOf(document = document),
            dueDate = dueDateOf(document = document, catalog = catalog),
        )
    }

    /** The top-level categories that hold any of [documents], by name, each with its subcategories' documents. */
    fun tiles(documents: Collection<DocumentModel>, catalog: CatalogModel): List<CategoryTileModel> {
        val byTop = documents.filter { it.categoryId != null }
            .groupBy { topOf(categoryId = checkNotNull(value = it.categoryId), catalog = catalog) }
        return byTop.mapNotNull { (topId, inside) ->
            val category = catalog.categories[topId] ?: return@mapNotNull null
            CategoryTileModel(
                categoryId = topId,
                name = category.name,
                count = inside.size,
                kinds = inside.sortedBy { it.path.value }.take(n = TILE_PAGES).map { it.kind },
            )
        }.sortedBy { it.name.lowercase(Locale.ROOT) }
    }

    fun counts(
        catalog: CatalogModel,
        reviewIds: Set<DocumentIdModel>,
        now: Instant,
        zone: ZoneId,
    ): SidebarCountsModel {
        val documents = catalog.documents.values
        val inbox = catalog.settings.layout.inbox
        val monthStart = LibraryFilter.startOfMonth(now = now, zone = zone)
        return SidebarCountsModel(
            all = documents.size,
            inbox = documents.count { inbox.contains(candidate = it.path) },
            needsReview = reviewIds.count { it in catalog.documents },
            addedThisMonth = documents.count { it.addedAt >= monthStart },
            folders = LibraryTrees.folders(documents = documents, foldersOnDisk = emptySet()).size - 1,
        )
    }

    /** The amount field, or else the first amount of money the document has. */
    private fun amountOf(document: DocumentModel): FieldValue.Money? {
        return document.fields[FieldPresets.AMOUNT.key] as? FieldValue.Money
            ?: document.fields.values.filterIsInstance<FieldValue.Money>().firstOrNull()
    }

    /** The first date of a field the library marks as a due date. */
    private fun dueDateOf(document: DocumentModel, catalog: CatalogModel): LocalDate? {
        return catalog.settings.fieldDefinitions
            .filter { it.role == FieldRole.DUE_DATE }
            .firstNotNullOfOrNull { (document.fields[it.key] as? FieldValue.Date)?.date }
    }

    private fun topOf(categoryId: CategoryIdModel, catalog: CatalogModel): CategoryIdModel {
        val visited = mutableSetOf<CategoryIdModel>()
        var current = categoryId
        while (visited.add(element = current)) {
            current = catalog.categories[current]?.parentId ?: return current
        }
        return current
    }
}
