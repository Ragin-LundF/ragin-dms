package dms.ui.library.state

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.library.LibraryFilter
import dms.ui.library.LibraryTrees
import dms.ui.library.documents.DocumentGroups
import dms.ui.library.documents.DocumentOrder
import dms.ui.library.filter.FacetOptions
import dms.ui.library.filter.FilterTokens
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FilterTokenModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.library.smart.model.SmartFolderEntryModel
import dms.ui.session.model.SessionStateModel
import java.time.Instant
import java.time.ZoneId

/**
 * Derives the library screen from the catalog and the user's choices. Pure, so the view model only
 * collects its inputs. The facet counts and the suggestions come from the documents of the sidebar's
 * view before the facet filters and the search text narrow them.
 */
object LibraryStates {

    fun of(
        catalog: CatalogModel?,
        session: SessionStateModel,
        view: LibraryViewStateModel,
        search: ContentSearchStateModel,
        smartMatches: Set<DocumentIdModel>?,
        selected: Set<DocumentIdModel>,
        reviewIds: Set<DocumentIdModel>,
        smartFolders: (CatalogModel) -> List<SmartFolderEntryModel>,
        now: Instant,
        zone: ZoneId,
    ): LibraryUiStateModel {
        val closed = LibraryUiStateModel(
            busy = session.busy,
            aiBusy = session.aiBusy,
            aiEnabled = session.aiEnabled,
            layout = view.layout,
            groupBy = view.groupBy,
            selection = view.selection,
        )
        if (catalog == null) {
            return closed
        }
        val inView = LibraryFilter.documents(
            catalog = catalog,
            view = view.copy(filters = emptyList(), query = ""),
            now = now,
            zone = zone,
            queryMatches = smartMatches,
            reviewIds = reviewIds,
        )
        val visible = LibraryFilter.documents(
            catalog = catalog,
            view = view,
            now = now,
            zone = zone,
            textMatches = search.matches,
            queryMatches = smartMatches,
            reviewIds = reviewIds,
        )
        return closed.copy(
            isOpen = true,
            title = LibraryFilter.titleOf(selection = view.selection, catalog = catalog),
            counts = DocumentRows.counts(catalog = catalog, reviewIds = reviewIds, now = now, zone = zone),
            categories = LibraryTrees.categories(
                categories = catalog.categories.values,
                documents = catalog.documents.values,
            ),
            tags = catalog.tags.sorted(),
            inspected = view.inspected?.let { catalog.documents[it] },
            collapsedGroups = view.collapsedGroups,
            contentSearch = search,
            smartFolders = smartFolders(catalog),
            sort = view.sort,
            primaryFolderId = session.folders?.primary?.id,
        )
            .withFilters(view = view, catalog = catalog, inView = inView, zone = zone)
            .withDocuments(view = view, catalog = catalog, visible = visible, selected = selected, zone = zone)
    }

    private fun LibraryUiStateModel.withFilters(
        view: LibraryViewStateModel,
        catalog: CatalogModel,
        inView: List<DocumentModel>,
        zone: ZoneId,
    ): LibraryUiStateModel {
        return copy(
            query = view.query,
            filters = view.filters,
            tokens = savedTokensOf(view = view, catalog = catalog) +
                view.filters.map { FilterTokens.tokenOf(filter = it, catalog = catalog) },
            suggestions = FilterTokens.suggestions(
                typed = view.query,
                documents = inView,
                active = view.filters,
                catalog = catalog,
            ),
            facets = FacetOptions.of(documents = inView, catalog = catalog, zone = zone),
            queryProblem = queryProblemOf(view = view),
        )
    }

    private fun LibraryUiStateModel.withDocuments(
        view: LibraryViewStateModel,
        catalog: CatalogModel,
        visible: List<DocumentModel>,
        selected: Set<DocumentIdModel>,
        zone: ZoneId,
    ): LibraryUiStateModel {
        val rows = DocumentOrder.sorted(
            rows = visible.map { DocumentRows.rowOf(document = it, catalog = catalog) },
            sort = view.sort,
        )
        val shownIds = visible.map { it.id }.toSet()
        val filtered = view.filters.isNotEmpty() || view.query.isNotBlank()
        return copy(
            tiles = if (view.selection.isAllDocuments() && !filtered) {
                DocumentRows.tiles(documents = catalog.documents.values, catalog = catalog)
            } else {
                emptyList()
            },
            documents = rows,
            groups = DocumentGroups.of(rows = rows, key = view.groupBy, zone = zone, catalog = catalog),
            selectedDocuments = selected.filterTo(destination = mutableSetOf()) { it in shownIds },
        )
    }

    private fun savedTokensOf(view: LibraryViewStateModel, catalog: CatalogModel): List<FilterTokenModel> {
        val saved = (view.selection as? LibrarySelection.SmartFolder)
            ?.let { selection -> catalog.settings.savedSearches.firstOrNull { it.id == selection.id } }
            ?: return emptyList()
        return FilterTokens.savedTokens(query = saved.query, catalog = catalog)
    }

    /** Why the view cannot become a saved view or an export, which both take it as one query. */
    private fun queryProblemOf(view: LibraryViewStateModel): String? {
        return when {
            view.filters.any { it is FacetFilter.Type } -> "A saved view or an export cannot keep the type filter yet"
            view.selection == LibrarySelection.NeedsReview -> "Needs review cannot be saved or exported as a view"
            else -> null
        }
    }

    private fun LibrarySelection.isAllDocuments(): Boolean {
        return this == LibrarySelection.Folder(path = RelativePathModel.ROOT)
    }
}
