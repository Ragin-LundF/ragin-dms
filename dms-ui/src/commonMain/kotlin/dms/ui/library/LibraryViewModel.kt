package dms.ui.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.catalog.model.CatalogEdit
import dms.application.common.OperationOutcome
import dms.application.document.model.AddDocumentsOutcome
import dms.application.port.storage.model.ExternalFileModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.taxonomy.model.CategoryModel
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.TextInputRequestModel
import dms.ui.export.ExportController
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.DocumentRowModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.library.search.ContentSearch
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.library.sidebar.SidebarWidth
import dms.ui.library.sidebar.model.ToggleTarget
import dms.ui.library.smart.SmartFolders
import dms.ui.library.taxonomy.TaxonomyController
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.NameInput
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.SessionStateModel
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.time.ZoneId

/**
 * The library: folders, categories and tags in the sidebar, the documents they select, adding files,
 * and dropping documents onto a folder, a category or a tag.
 */
class LibraryViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
) : ViewModel() {

    private val view = MutableStateFlow(value = LibraryViewStateModel())

    val dialogs: DialogController = DialogController()

    val sidebarWidth: SidebarWidth = SidebarWidth()

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

    /** Editing the categories and tags of the sidebar. */
    val taxonomy: TaxonomyController = TaxonomyController(
        services = services,
        session = session,
        dialogs = dialogs,
        select = ::select,
    )

    val export: ExportController = ExportController(
        services = services,
        session = session,
        platform = platform,
        scope = viewModelScope,
        currentQuery = ::currentQuery,
    )

    val state: StateFlow<LibraryUiStateModel> = combine(
        flow = services.library.catalog,
        flow2 = session.state,
        flow3 = view,
        flow4 = contentSearch.state,
        flow5 = smartFolders.textMatches,
    ) { catalog, sessionState, viewState, search, smartMatches ->
        derive(catalog = catalog, sessionState = sessionState, view = viewState, search = search)
            .let { derived -> if (smartMatches == null) derived else derived.onlyMatching(ids = smartMatches) }
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = LibraryUiStateModel())

    fun select(selection: LibrarySelection) {
        view.update { it.copy(selection = selection).unfolding(selection = selection) }
    }

    /** Folds or unfolds a folder, a category or a whole section of the sidebar. */
    fun toggle(target: ToggleTarget) {
        view.update { state ->
            when (target) {
                is ToggleTarget.Folder -> state.copy(expandedFolders = state.expandedFolders.flip(target.path))
                is ToggleTarget.Category -> state.copy(
                    expandedCategories = state.expandedCategories.flip(target.categoryId),
                )

                is ToggleTarget.Group -> state.copy(collapsedGroups = state.collapsedGroups.flip(target.group))
            }
        }
    }

    /** Shows only documents of [year]; `null` shows every year. */
    fun filterYear(year: YearFilterModel?) {
        view.update { it.copy(year = year) }
    }

    fun search(query: String) {
        view.update { it.copy(query = query) }
    }

    fun inspect(documentId: DocumentIdModel?) {
        view.update { it.copy(inspected = documentId) }
    }

    fun setLayout(layout: DocumentLayout) {
        view.update { it.copy(layout = layout) }
    }

    /** Shows the documents of [selection] that match [query] and [year], as the dashboard asks. */
    fun show(selection: LibrarySelection, query: String, year: YearFilterModel? = null) {
        view.update {
            it.copy(selection = selection, query = query, year = year, inspected = null)
                .unfolding(selection = selection)
        }
    }

    /** Shows every document with [documentId] open in the inspector. */
    fun reveal(documentId: DocumentIdModel) {
        view.update {
            it.copy(
                selection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
                query = "",
                year = null,
                inspected = documentId,
            )
        }
    }

    /** Asks for files and copies them into the selected folder of every library folder. */
    fun addFiles() {
        chooseAndAdd(target = selectedFolder())
    }

    /** Asks for files and copies them into the top level of every library folder. */
    fun addFilesToTopLevel() {
        chooseAndAdd(target = RelativePathModel.ROOT)
    }

    fun addFiles(locations: List<String>, target: RelativePathModel = selectedFolder()) {
        if (locations.isEmpty()) {
            session.report(message = UiMessageModel(text = "Only files can be added, not folders", isError = true))
            return
        }
        session.perform(label = "Adding ${locations.size} file(s)") {
            val outcome = services.documents.addDocuments(
                files = locations.map { ExternalFileModel(location = it) },
                targetFolder = target,
            )
            val added = when (outcome) {
                is AddDocumentsOutcome.Added -> outcome.files
                AddDocumentsOutcome.LibraryNotOpen -> {
                    session.report(message = OutcomeMessages.of(outcome = OperationOutcome.LibraryNotOpen, done = ""))
                    return@perform
                }
            }
            val done = added.count { it.outcome is OperationOutcome.Done }
            val duplicates = added.count { it.duplicates.isNotEmpty() }
            val problem = added.firstOrNull { it.outcome !is OperationOutcome.Done }
                ?.let { "${it.source.fileName}: ${OutcomeMessages.of(outcome = it.outcome, done = "").text}" }
            val text = listOfNotNull(
                "Added $done of ${locations.size} file(s)",
                duplicates.takeIf { it > 0 }?.let { "$it already in the library under another name" },
                problem,
            ).joinToString(separator = "; ")
            session.report(message = UiMessageModel(text = text, isError = done < locations.size))
        }
    }

    /**
     * Applies a drop: a document moves into a folder or gets a category or a tag; a category moves into
     * another category or to the top level.
     */
    fun drop(item: DragItem, target: DropTarget) {
        when (item) {
            is DragItem.DraggedDocument -> dropDocument(document = item.document, target = target)
            is DragItem.DraggedCategory -> dropCategory(category = item.category, target = target)
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

    private fun dropDocument(document: DocumentModel, target: DropTarget) {
        val name = document.path.fileName
        when (target) {
            is DropTarget.Folder -> if ((document.path.parent ?: RelativePathModel.ROOT) != target.path) {
                val where = target.path.value.ifEmpty { "the top level" }
                session.perform(label = "Moving $name") {
                    val outcome = services.documents.moveDocument(
                        documentId = document.id,
                        newPath = target.path.resolve(name = name),
                    )
                    session.report(message = OutcomeMessages.of(outcome = outcome, done = "Moved $name to $where"))
                }
            }

            is DropTarget.Category -> if (document.categoryId != target.categoryId) {
                val category = services.library.catalog.value?.categories?.get(target.categoryId)?.name
                val edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = target.categoryId)
                taxonomy.edit(edit = edit, done = "Filed $name under $category")
            }

            DropTarget.Uncategorized -> if (document.categoryId != null) {
                val edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = null)
                taxonomy.edit(edit = edit, done = "Removed the category of $name")
            }

            is DropTarget.Tag -> if (target.tag !in document.tags) {
                val edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(target.tag))
                taxonomy.edit(edit = edit, done = "Tagged $name with ${target.tag.value}")
            }

            DropTarget.TopLevelCategory -> Unit
        }
    }

    fun newFolder() {
        val parent = selectedFolder()
        val title = "New folder in ${parent.value.ifEmpty { "the top level" }}"
        askName(title = title, initial = "", confirm = "Create") { name ->
            val path = NameInput.child(parent = parent, raw = name)
            session.perform(label = "Creating folder ${path.value}") {
                val outcome = services.documents.createFolder(path = path)
                if (outcome is OperationOutcome.Done) {
                    view.update { state ->
                        state.copy(
                            createdFolders = state.createdFolders + path,
                            selection = LibrarySelection.Folder(path = path),
                        )
                    }
                }
                session.report(message = OutcomeMessages.of(outcome = outcome, done = "Created ${path.value}"))
            }
        }
    }

    fun renameFolder(path: RelativePathModel) {
        askName(title = "Rename folder", initial = path.fileName, confirm = "Rename") { name ->
            val target = NameInput.child(parent = path.parent ?: RelativePathModel.ROOT, raw = name)
            session.perform(label = "Renaming folder ${path.value}") {
                val outcome = services.documents.moveFolder(from = path, to = target)
                if (outcome is OperationOutcome.Done) {
                    view.update { state ->
                        state.copy(
                            createdFolders = state.createdFolders
                                .map { relocated(path = it, from = path, to = target) }
                                .toSet(),
                            selection = LibrarySelection.Folder(path = target),
                        )
                    }
                }
                session.report(message = OutcomeMessages.of(outcome = outcome, done = "Renamed to ${target.value}"))
            }
        }
    }

    /** A new category below the selected one, or at the top level. */
    fun newCategory() {
        taxonomy.newCategory(parentId = (view.value.selection as? LibrarySelection.Category)?.categoryId)
    }

    private fun chooseAndAdd(target: RelativePathModel) {
        viewModelScope.launch {
            val files = platform.chooseFiles(title = "Add files to the library")
            if (files.isNotEmpty()) {
                addFiles(locations = files, target = target)
            }
        }
    }

    /** Asks for a file or folder name and passes it on only if the file name policy allows it. */
    private fun askName(title: String, initial: String, confirm: String, submit: (String) -> Unit) {
        val request = TextInputRequestModel(
            title = title,
            label = "Name",
            initialValue = initial,
            confirmLabel = confirm,
        )
        dialogs.askText(request = request) { name ->
            val problem = NameInput.problem(raw = name)
            if (problem == null) {
                submit(name)
            } else {
                session.report(message = UiMessageModel(text = problem, isError = true))
            }
        }
    }

    /** What the library shows now as one query: selection, year, search text and, if on, its content term. */
    private fun currentQuery(): DocumentQueryModel {
        val current = view.value
        val catalog = services.library.catalog.value ?: return DocumentQueryModel.ALL
        val base = LibraryFilter.queryOf(
            selection = current.selection,
            year = current.year,
            now = services.library.clock.now(),
            catalog = catalog,
        ) ?: DocumentQueryModel.ALL
        val text = current.query.trim().ifEmpty { null }
        return if (contentSearch.state.value.enabled) base.copy(contentText = text) else base.copy(text = text)
    }

    private fun LibraryUiStateModel.onlyMatching(ids: Set<DocumentIdModel>): LibraryUiStateModel {
        return copy(documents = documents.filter { it.document.id in ids })
    }

    private fun selectedFolder(): RelativePathModel {
        return (view.value.selection as? LibrarySelection.Folder)?.path ?: RelativePathModel.ROOT
    }

    private fun relocated(path: RelativePathModel, from: RelativePathModel, to: RelativePathModel): RelativePathModel {
        val moves = path == from || path.isInside(folder = from)
        return if (moves) path.relocate(from = from, to = to) else path
    }

    private fun derive(
        catalog: CatalogModel?,
        sessionState: SessionStateModel,
        view: LibraryViewStateModel,
        search: ContentSearchStateModel,
    ): LibraryUiStateModel {
        if (catalog == null) {
            return LibraryUiStateModel(busy = sessionState.busy, aiBusy = sessionState.aiBusy)
        }
        val documents = catalog.documents.values
        val zone = ZoneId.systemDefault()
        val visible = LibraryFilter.documents(
            catalog = catalog,
            view = view,
            now = services.library.clock.now(),
            zone = zone,
            textMatches = search.matches,
        )
        return LibraryUiStateModel(
            isOpen = true,
            title = LibraryFilter.titleOf(selection = view.selection, catalog = catalog),
            folders = LibraryTrees.folders(
                documents = documents,
                createdFolders = view.createdFolders,
                expanded = view.expandedFolders,
            ),
            categories = LibraryTrees.categories(
                categories = catalog.categories.values,
                documents = documents,
                expanded = view.expandedCategories,
            ),
            uncategorizedCount = documents.count { it.categoryId == null },
            tags = catalog.tags.sorted(),
            tagCounts = documents.flatMap { it.tags }.groupingBy { it }.eachCount(),
            selection = view.selection,
            query = view.query,
            documents = visible.map { document ->
                DocumentRowModel(
                    document = document,
                    categoryName = document.categoryId?.let { catalog.categories[it]?.name },
                )
            },
            inspected = view.inspected?.let { catalog.documents[it] },
            busy = sessionState.busy,
            aiBusy = sessionState.aiBusy,
            layout = view.layout,
            years = LibraryFilter.years(documents = documents, zone = zone),
            year = view.year,
            collapsedGroups = view.collapsedGroups,
            contentSearch = search,
            smartFolders = smartFolders.entriesOf(catalog = catalog),
        )
    }

    private fun <T> Set<T>.flip(item: T): Set<T> {
        return if (item in this) this - item else this + item
    }

    /** The view with the folders and categories above [selection] unfolded, so the selection shows. */
    private fun LibraryViewStateModel.unfolding(selection: LibrarySelection): LibraryViewStateModel {
        return when (selection) {
            is LibrarySelection.Folder -> copy(
                expandedFolders = expandedFolders + generateSequence(seed = selection.path.parent) { it.parent }
                    .filterNot { it.isRoot },
            )

            is LibrarySelection.Category -> {
                val categories = services.library.catalog.value?.categories.orEmpty()
                val parents = generateSequence(seed = categories[selection.categoryId]?.parentId) { id ->
                    categories[id]?.parentId
                }.take(n = categories.size)
                copy(expandedCategories = expandedCategories + parents)
            }

            else -> this
        }
    }
}
