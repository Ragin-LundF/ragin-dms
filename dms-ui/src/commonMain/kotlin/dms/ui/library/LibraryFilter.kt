package dms.ui.library

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.QueryContextModel
import dms.domain.query.model.TagFilterModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.model.YearFilterModel
import java.time.Duration
import java.time.Instant
import java.time.ZoneId
import java.util.Locale

/** Which documents the library shows: the sidebar's selection, the year and the text filter together. */
object LibraryFilter {

    /** How far back "Recently added" reaches. */
    private val RECENT: Duration = Duration.ofDays(30)

    /**
     * The documents [view] selects; recent ones newest first, all others by path. The query matches
     * the name, title, correspondent and tags, and finds [textMatches] as well.
     */
    fun documents(
        catalog: CatalogModel,
        view: LibraryViewStateModel,
        now: Instant,
        zone: ZoneId,
        textMatches: Set<DocumentIdModel> = emptySet(),
        queryMatches: Set<DocumentIdModel>? = null,
    ): List<DocumentModel> {
        val context = QueryContextModel(catalog = catalog, today = now.atZone(zone).toLocalDate(), zone = zone)
        val selected = queryOf(selection = view.selection, year = view.year, now = now, catalog = catalog)
            ?: return emptyList()
        val text = DocumentQueryModel(text = view.query.trim().lowercase(Locale.ROOT))
        val shown = catalog.documents.values.filter { document ->
            selected.matches(document = document, context = context) &&
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

    fun titleOf(selection: LibrarySelection, catalog: CatalogModel): String {
        return when (selection) {
            is LibrarySelection.Folder -> if (selection.path.isRoot) "All documents" else selection.path.fileName
            is LibrarySelection.Category -> catalog.categories[selection.categoryId]?.name ?: "Category"
            LibrarySelection.Uncategorized -> "Uncategorized"
            is LibrarySelection.Tag -> "#${selection.tag.value}"
            LibrarySelection.Recent -> "Recently added"
            is LibrarySelection.SmartFolder ->
                catalog.settings.savedSearches.firstOrNull { it.id == selection.id }?.name ?: "Smart folder"
        }
    }

    /**
     * The sidebar's [selection] and the [year] as one query, so the library, smart folders and every
     * other place that selects documents share one set of rules. A category includes its subcategories.
     * `null` for a smart folder the catalog no longer has.
     */
    fun queryOf(
        selection: LibrarySelection,
        year: YearFilterModel?,
        now: Instant,
        catalog: CatalogModel,
    ): DocumentQueryModel? {
        val dates = year?.let {
            if (it.andEarlier) DateRangeModel.untilEndOf(year = it.year) else DateRangeModel.year(year = it.year)
        }
        val query = DocumentQueryModel(dates = dates)
        return when (selection) {
            is LibrarySelection.Folder -> query.copy(folder = selection.path)
            is LibrarySelection.Category -> query.copy(category = selection.categoryId)
            LibrarySelection.Uncategorized -> query.copy(uncategorized = true)
            is LibrarySelection.Tag -> query.copy(tags = TagFilterModel(all = setOf(selection.tag)))
            LibrarySelection.Recent -> query.copy(addedSince = now.minus(RECENT))
            is LibrarySelection.SmartFolder -> catalog.settings.savedSearches.firstOrNull { it.id == selection.id }
                ?.query
                ?.let { saved -> if (dates == null) saved else saved.copy(dates = dates) }
        }
    }
}
