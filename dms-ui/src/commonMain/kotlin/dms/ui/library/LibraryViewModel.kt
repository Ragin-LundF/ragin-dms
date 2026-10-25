package dms.ui.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.taxonomy.model.CategoryModel
import dms.ui.PlatformActions
import dms.ui.library.adding.FileAdder
import dms.ui.components.dialog.DialogController
import dms.ui.document.quicklook.QuickLookController
import dms.ui.export.ExportController
import dms.ui.library.documents.DocumentOrder
import dms.ui.library.documents.model.DocumentClick
import dms.ui.library.documents.model.GroupKey
import dms.ui.library.documents.model.SortColumn
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.filter.FilterTokens
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.folders.FolderBrowser
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.library.search.ContentSearch
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.library.selection.BulkActions
import dms.ui.library.selection.DocumentSelection
import dms.ui.library.selection.model.DocumentSelectionModel
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.smart.SmartFolders
import dms.ui.library.state.DiskFolders
import dms.ui.library.state.LibraryStates
import dms.ui.library.state.PanelWidth
import dms.ui.library.state.PendingReviews
import dms.ui.library.state.model.LayoutPanel
import dms.ui.library.taxonomy.TaxonomyController
import dms.ui.preview.ThumbnailLoader
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.time.ZoneId

/**
 * The library: the sidebar's views, the search with its facet filters, the documents grouped in a grid
 * or a list, ticking documents for the bulk bar, adding files, and dropping documents onto a folder.
 */
class LibraryViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
    val quickLook: QuickLookController = QuickLookController(),
) : ViewModel() {

    private val view = MutableStateFlow(value = LibraryViewStateModel())
    private val searchFocus = MutableStateFlow(value = 0)

    val dialogs: DialogController = DialogController()

    /** The open catalog, for the rows the folder view's cards show. */
    val catalog: StateFlow<CatalogModel?> = services.library.catalog

    val sidebarWidth: PanelWidth = PanelWidth(
        panel = LayoutPanel.SIDEBAR,
        layout = session.layout,
        stored = session.state.map { it.preferences.layout }.distinctUntilChanged(),
        current = { session.state.value.preferences.layout },
        scope = viewModelScope,
    )

    val contentSearch: ContentSearch = ContentSearch(
        services = services,
        queries = view.map { it.query },
        scope = viewModelScope,
    )

    val smartFolders: SmartFolders = SmartFolders(
        services = services,
        session = session,
        dialogs = dialogs,
        currentQuery = ::currentQuery,
        selected = view.map { (it.selection as? LibrarySelection.SmartFolder)?.id },
        scope = viewModelScope,
    )

    /** Creating, renaming and deleting categories and tags, from the facet popovers. */
    val taxonomy: TaxonomyController = TaxonomyController(
        services = services,
        session = session,
        dialogs = dialogs,
        select = ::select,
    )

    val thumbnails: ThumbnailLoader = ThumbnailLoader(services = services, platform = platform)

    val export: ExportController = ExportController(
        services = services,
        session = session,
        platform = platform,
        scope = viewModelScope,
        currentQuery = ::currentQuery,
    )

    /** The folders on disk, also empty ones, for the folder view and both "Move to…" pickers. */
    private val diskFolders = DiskFolders(services = services, session = session, scope = viewModelScope)

    /** The folder view behind "Folders": the tree, the open folder and its folder actions. */
    val folders: FolderBrowser = FolderBrowser(
        services = services,
        session = session,
        dialogs = dialogs,
        disk = diskFolders,
        scope = viewModelScope,
    )

    /** The ticked documents; ⌘A and Esc go to [DocumentSelection.selectAll] and [DocumentSelection.clear]. */
    val documentSelection: DocumentSelection = DocumentSelection(shown = ::shownIds)

    /** The actions of the bulk bar on the ticked documents. */
    val bulk: BulkActions = BulkActions(
        services = services,
        session = session,
        dialogs = dialogs,
        chosen = ::chosenDocuments,
        foldersOnDisk = { diskFolders.folders.value },
        done = { documentSelection.clear() },
    )

    /** Adding files, chosen in a dialog or dropped, to the filtered folder or a given one. */
    val files: FileAdder = FileAdder(
        services = services,
        session = session,
        platform = platform,
        scope = viewModelScope,
        filteredFolder = ::filteredFolder,
    )

    /** Grows each time something asks to focus the search field, which then takes the focus. */
    val searchFocusRequests: StateFlow<Int> = searchFocus.asStateFlow()

    private val pendingReviews = PendingReviews(session = session, scope = viewModelScope)

    private val matches = combine(flow = contentSearch.state, flow2 = smartFolders.textMatches) { search, smart ->
        TextMatches(search = search, smart = smart)
    }

    private val choices = combine(
        flow = view,
        flow2 = documentSelection.state,
        flow3 = pendingReviews.ids,
    ) { viewState, selection, reviewIds -> Choices(view = viewState, selection = selection, reviewIds = reviewIds) }

    val state: StateFlow<LibraryUiStateModel> = combine(
        flow = services.library.catalog,
        flow2 = session.state,
        flow3 = matches,
        flow4 = choices,
    ) { catalog, sessionState, textMatches, choice ->
        LibraryStates.of(
            catalog = catalog,
            session = sessionState,
            view = choice.view,
            search = textMatches.search,
            smartMatches = textMatches.smart,
            selected = choice.selection.selected,
            reviewIds = choice.reviewIds,
            smartFolders = smartFolders::entriesOf,
            now = services.library.clock.now(),
            zone = ZoneId.systemDefault(),
        )
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = LibraryUiStateModel())

    init {
        // Quick look decides which document is shown; the view only mirrors it for the rings and rows.
        viewModelScope.launch {
            quickLook.state.map { it.shown }.distinctUntilChanged().collect { shown ->
                view.update { it.copy(inspected = shown) }
            }
        }
        viewModelScope.launch {
            state.map { current -> current.documents.map { it.document.id } }.distinctUntilChanged().collect { order ->
                quickLook.updateOrder(order = order)
            }
        }
    }

    /**
     * Opens [selection]: a sidebar view without filters, or "All documents" filtered by the category,
     * tag or folder that [selection] names. The search text and the ticks are cleared.
     */
    fun select(selection: LibrarySelection) {
        view.update { it.opening(selection = selection) }
        documentSelection.clear()
        quickLook.close()
    }

    /** Shows the documents of [selection] that match [query] and [year], as the dashboard and the palette ask. */
    fun show(selection: LibrarySelection, query: String, year: YearFilterModel? = null) {
        view.update { state ->
            val opened = state.opening(selection = selection)
            val withYear = year?.let { FacetFilter.Year(year = it) }
                ?.let { FilterTokens.with(filters = opened.filters, filter = it) }
            opened.copy(query = query, filters = withYear ?: opened.filters)
        }
        documentSelection.clear()
        quickLook.close()
    }

    /** Shows every document with [documentId] open in quick look. */
    fun reveal(documentId: DocumentIdModel) {
        view.update { state -> state.opening(selection = LibrarySelection.Folder(path = RelativePathModel.ROOT)) }
        documentSelection.clear()
        quickLook.open(documentId = documentId, order = shownIds())
    }

    /** Folds or unfolds a section of the sidebar. */
    fun toggleSection(group: SidebarGroup) {
        view.update { state ->
            val collapsed = state.collapsedGroups
            state.copy(collapsedGroups = if (group in collapsed) collapsed - group else collapsed + group)
        }
    }

    fun search(query: String) {
        view.update { it.copy(query = query) }
    }

    /** Applies [filter] in place of the filter its facet had; a suggestion taken from the search clears the text. */
    fun addFilter(filter: FacetFilter, clearQuery: Boolean = false) {
        view.update { state ->
            state.copy(
                filters = FilterTokens.with(filters = state.filters, filter = filter),
                query = if (clearQuery) "" else state.query,
            )
        }
    }

    fun removeFilter(filter: FacetFilter) {
        view.update { it.copy(filters = it.filters - filter) }
    }

    /** Removes every facet filter and the search text. */
    fun clearFilters() {
        view.update { it.copy(filters = emptyList(), query = "") }
    }

    fun groupBy(key: GroupKey) {
        view.update { it.copy(groupBy = key) }
    }

    /** Sorts the documents by [column]; a second click on the same column turns the order around. */
    fun sortBy(column: SortColumn) {
        view.update { it.copy(sort = DocumentOrder.next(current = it.sort, column = column)) }
    }

    /**
     * A plain click opens [documentId] in quick look, or closes it when it is open; ⌘/Ctrl ticks or
     * unticks it, Shift ticks every document from the last ticked one to it.
     */
    fun clickDocument(documentId: DocumentIdModel, click: DocumentClick) {
        when (click) {
            DocumentClick.PLAIN -> inspect(documentId = documentId.takeUnless { it == quickLook.state.value.shown })
            DocumentClick.TOGGLE -> documentSelection.toggle(documentId = documentId)
            DocumentClick.RANGE -> documentSelection.extend(documentId = documentId)
        }
    }

    /** Opens [documentId] in quick look, in the order the documents are shown; `null` closes it. */
    fun inspect(documentId: DocumentIdModel?) {
        if (documentId == null) {
            quickLook.close()
        } else {
            quickLook.open(documentId = documentId, order = shownIds())
        }
    }

    fun setLayout(layout: DocumentLayout) {
        view.update { it.copy(layout = layout) }
    }

    /** Puts the focus into the search field (/). */
    fun focusSearch() {
        searchFocus.update { it + 1 }
    }

    /**
     * Applies a drop. A document moves into a folder or gets a category or a tag; when it is ticked, the
     * whole selection goes with it. A category moves into another category or to the top level.
     */
    fun drop(item: DragItem, target: DropTarget) {
        when (item) {
            is DragItem.DraggedDocument -> dropDocuments(
                documents = draggedWith(document = item.document),
                target = target,
            )

            is DragItem.DraggedCategory -> dropCategory(category = item.category, target = target)
            is DragItem.DraggedFolder -> when (target) {
                is DropTarget.Folder -> folders.editor.moveInto(path = item.path, folder = target.path)
                is DropTarget.Category,
                DropTarget.TopLevelCategory,
                DropTarget.Uncategorized,
                is DropTarget.Tag,
                -> Unit
            }
        }
    }

    private fun dropDocuments(documents: List<DocumentModel>, target: DropTarget) {
        when (target) {
            is DropTarget.Folder -> bulk.moveTo(target = target.path, documents = documents)
            is DropTarget.Category -> bulk.assignCategory(categoryId = target.categoryId, documents = documents)
            DropTarget.Uncategorized -> bulk.assignCategory(categoryId = null, documents = documents)
            is DropTarget.Tag -> bulk.addTag(tag = target.tag, documents = documents)
            DropTarget.TopLevelCategory -> Unit
        }
    }

    private fun dropCategory(category: CategoryModel, target: DropTarget) {
        val parentId = (target as? DropTarget.Category)?.categoryId
        val movable = target is DropTarget.Category || target == DropTarget.TopLevelCategory
        if (!movable || parentId == category.parentId || parentId == category.id) {
            return
        }
        taxonomy.moveCategory(category = category, parentId = parentId)
    }

    /** The dragged [document], with every other ticked document when it is ticked itself. */
    private fun draggedWith(document: DocumentModel): List<DocumentModel> {
        return if (document.id in state.value.selectedDocuments) chosenDocuments() else listOf(document)
    }

    /** What the library shows now as one query: view, filters, search text and, if on, its content term. */
    private fun currentQuery(): DocumentQueryModel {
        val current = view.value
        val catalog = services.library.catalog.value ?: return DocumentQueryModel.ALL
        val base = LibraryFilter.queryOf(
            selection = current.selection,
            filters = current.filters,
            now = services.library.clock.now(),
            zone = ZoneId.systemDefault(),
            catalog = catalog,
        ) ?: DocumentQueryModel.ALL
        val text = current.query.trim().ifEmpty { null }
        return if (contentSearch.state.value.enabled) base.copy(contentText = text) else base.copy(text = text)
    }

    private fun filteredFolder(): RelativePathModel {
        return view.value.filters.filterIsInstance<FacetFilter.Folder>().firstOrNull()?.path ?: RelativePathModel.ROOT
    }

    private fun shownIds(): List<DocumentIdModel> {
        return state.value.documents.map { it.document.id }
    }

    /** The ticked documents, in the order they are shown. */
    private fun chosenDocuments(): List<DocumentModel> {
        val current = state.value
        return current.documents.map { it.document }.filter { it.id in current.selectedDocuments }
    }

    /** The view opening [selection], with the filters a category, tag or folder below the root stands for. */
    private fun LibraryViewStateModel.opening(selection: LibrarySelection): LibraryViewStateModel {
        val all = LibrarySelection.Folder(path = RelativePathModel.ROOT)
        val (shown, filter) = when (selection) {
            is LibrarySelection.Folder -> all to selection.path.takeUnless { it.isRoot }
                ?.let { FacetFilter.Folder(path = it) }

            is LibrarySelection.Category -> all to FacetFilter.Category(categoryId = selection.categoryId)
            LibrarySelection.Uncategorized -> all to FacetFilter.Category(categoryId = null)
            is LibrarySelection.Tag -> all to FacetFilter.Tag(tag = selection.tag)
            LibrarySelection.Inbox,
            LibrarySelection.NeedsReview,
            LibrarySelection.Recent,
            LibrarySelection.Folders,
            is LibrarySelection.SmartFolder,
            -> selection to null
        }
        return copy(selection = shown, filters = listOfNotNull(filter), query = "")
    }

    /** The content search's state and the matches of a saved view that needs the documents' text. */
    private data class TextMatches(val search: ContentSearchStateModel, val smart: Set<DocumentIdModel>?)

    /** What the user chose, the ticks, and the documents waiting for review. */
    private data class Choices(
        val view: LibraryViewStateModel,
        val selection: DocumentSelectionModel,
        val reviewIds: Set<DocumentIdModel>,
    )
}
