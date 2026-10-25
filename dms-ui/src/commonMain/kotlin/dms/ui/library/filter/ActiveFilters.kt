package dms.ui.library.filter

import dms.domain.catalog.model.CatalogModel
import dms.ui.library.filter.model.ActiveFilter
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryViewStateModel

/**
 * The filters the library applies now, as chips: a tag or category chosen in the sidebar, the year
 * and the search text. A folder, "Uncategorized", "Recently added" and a smart folder are places in
 * the sidebar, not filters.
 */
object ActiveFilters {

    fun of(view: LibraryViewStateModel, catalog: CatalogModel, contentSearch: Boolean): List<ActiveFilter> {
        return listOfNotNull(
            selectionFilter(selection = view.selection, catalog = catalog),
            view.year?.let { ActiveFilter.Year(year = it) },
            view.query.trim().takeIf { it.isNotEmpty() }?.let { query ->
                ActiveFilter.Text(query = query, inContent = contentSearch)
            },
        )
    }

    fun label(filter: ActiveFilter): String {
        return when (filter) {
            is ActiveFilter.Tag -> "tag: ${filter.tag.value}"
            is ActiveFilter.Category -> "category: ${filter.name}"
            is ActiveFilter.Year -> "year: ${filter.year.label}"
            is ActiveFilter.Text -> {
                val where = if (filter.inContent) "text" else "names"
                "\"${filter.query}\" in $where"
            }
        }
    }

    private fun selectionFilter(selection: LibrarySelection, catalog: CatalogModel): ActiveFilter? {
        return when (selection) {
            is LibrarySelection.Tag -> ActiveFilter.Tag(tag = selection.tag)
            is LibrarySelection.Category -> catalog.categories[selection.categoryId]?.let { category ->
                ActiveFilter.Category(categoryId = category.id, name = category.name)
            }

            is LibrarySelection.Folder,
            LibrarySelection.Uncategorized,
            LibrarySelection.Recent,
            is LibrarySelection.SmartFolder,
            -> null
        }
    }
}
