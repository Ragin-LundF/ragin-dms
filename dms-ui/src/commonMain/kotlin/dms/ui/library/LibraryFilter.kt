package dms.ui.library

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.QueryContextModel
import dms.domain.query.model.TagFilterModel
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.model.YearFilterModel
import java.time.Instant
import java.time.ZoneId
import java.util.Locale

/**
 * Which documents the library shows: the sidebar's view, the facet filters and the search text
 * together. The one place that decides it, so the list, the counts, saving a view and exporting agree.
 */
object LibraryFilter {

    /**
     * The documents [view] shows; "Added this month" newest first, all others by path. The search text
     * matches the name, title, correspondent and tags, and finds [textMatches] as well. "Needs review"
     * shows only [reviewIds].
     */
    fun documents(
        catalog: CatalogModel,
        view: LibraryViewStateModel,
        now: Instant,
        zone: ZoneId,
        textMatches: Set<DocumentIdModel> = emptySet(),
        queryMatches: Set<DocumentIdModel>? = null,
        reviewIds: Set<DocumentIdModel> = emptySet(),
    ): List<DocumentModel> {
        val context = QueryContextModel(catalog = catalog, today = now.atZone(zone).toLocalDate(), zone = zone)
        val selected = queryOf(
            selection = view.selection,
            filters = view.filters,
            now = now,
            zone = zone,
            catalog = catalog,
        ) ?: return emptyList()
        val kind = view.filters.filterIsInstance<FacetFilter.Type>().firstOrNull()?.kind
        val text = DocumentQueryModel(text = view.query.trim().lowercase(Locale.ROOT))
        val shown = catalog.documents.values.filter { document ->
            selected.matches(document = document, context = context) &&
                (kind == null || document.kind == kind) &&
                (view.selection != LibrarySelection.NeedsReview || document.id in reviewIds) &&
                (queryMatches == null || document.id in queryMatches) &&
                (text.matches(document = document, context = context) || document.id in textMatches)
        }
        return if (view.selection == LibrarySelection.Recent) {
            shown.sortedByDescending { it.addedAt }
        } else {
            shown.sortedBy { it.path.value.lowercase(Locale.ROOT) }
        }
    }

    /** The year a document belongs to: of its document date, or of when it was added. */
    fun yearOf(document: DocumentModel, zone: ZoneId): Int {
        return document.metadata.documentDate?.year ?: document.addedAt.atZone(zone).year
    }

    /** Every year a document belongs to, newest first. */
    fun years(documents: Collection<DocumentModel>, zone: ZoneId): List<Int> {
        return documents.map { yearOf(document = it, zone = zone) }.distinct().sortedDescending()
    }

    /** Where "Added this month" begins: the first day of this calendar month in [zone]. */
    fun startOfMonth(now: Instant, zone: ZoneId): Instant {
        return now.atZone(zone).toLocalDate().withDayOfMonth(1).atStartOfDay(zone).toInstant()
    }

    fun titleOf(selection: LibrarySelection, catalog: CatalogModel): String {
        return when (selection) {
            is LibrarySelection.Folder -> if (selection.path.isRoot) "All documents" else selection.path.fileName
            is LibrarySelection.Category -> catalog.categories[selection.categoryId]?.name ?: "Category"
            LibrarySelection.Uncategorized -> "Without a category"
            is LibrarySelection.Tag -> "#${selection.tag.value}"
            LibrarySelection.Inbox -> "Inbox"
            LibrarySelection.NeedsReview -> "Needs review"
            LibrarySelection.Recent -> "Added this month"
            LibrarySelection.Folders -> "Folders"
            is LibrarySelection.SmartFolder ->
                catalog.settings.savedSearches.firstOrNull { it.id == selection.id }?.name ?: "Saved view"
        }
    }

    /**
     * The view [selection] with [filters] as one query, so the library, saved views, exports and every
     * other place that selects documents share one set of rules. A category includes its subcategories.
     * `null` where the view shows no documents: a saved view the catalog no longer has, the inbox while
     * it is off, and the folder view, which lists its documents itself. The type filter and "Needs
     * review" cannot be part of a query; [documents] applies them on top.
     */
    fun queryOf(
        selection: LibrarySelection,
        filters: List<FacetFilter>,
        now: Instant,
        zone: ZoneId,
        catalog: CatalogModel,
    ): DocumentQueryModel? {
        val base = baseQueryOf(selection = selection, now = now, zone = zone, catalog = catalog) ?: return null
        return filters.fold(initial = base) { query, filter -> query.narrowedBy(filter = filter) }
    }

    private fun baseQueryOf(
        selection: LibrarySelection,
        now: Instant,
        zone: ZoneId,
        catalog: CatalogModel,
    ): DocumentQueryModel? {
        val query = DocumentQueryModel()
        return when (selection) {
            is LibrarySelection.Folder -> query.copy(folder = selection.path)
            is LibrarySelection.Category -> query.copy(category = selection.categoryId)
            LibrarySelection.Uncategorized -> query.copy(uncategorized = true)
            is LibrarySelection.Tag -> query.copy(tags = TagFilterModel(all = setOf(selection.tag)))
            LibrarySelection.Inbox -> catalog.settings.layout.inbox.takeIf { it.enabled }
                ?.let { query.copy(folder = it.path) }

            LibrarySelection.NeedsReview -> query
            LibrarySelection.Recent -> query.copy(addedSince = startOfMonth(now = now, zone = zone))
            LibrarySelection.Folders -> null
            is LibrarySelection.SmartFolder -> catalog.settings.savedSearches.firstOrNull { it.id == selection.id }
                ?.query
        }
    }

    private fun DocumentQueryModel.narrowedBy(filter: FacetFilter): DocumentQueryModel {
        return when (filter) {
            is FacetFilter.Category -> copy(category = filter.categoryId, uncategorized = filter.categoryId == null)
            is FacetFilter.Tag -> copy(tags = tags.copy(all = tags.all + filter.tag))
            is FacetFilter.Year -> copy(dates = datesOf(year = filter.year))
            is FacetFilter.Correspondent -> copy(correspondent = filter.name)
            is FacetFilter.Type -> this
            is FacetFilter.Folder -> copy(folder = filter.path)
        }
    }

    private fun datesOf(year: YearFilterModel): DateRangeModel {
        return if (year.andEarlier) {
            DateRangeModel.untilEndOf(year = year.year)
        } else {
            DateRangeModel.year(year = year.year)
        }
    }
}
