package dms.ui.library

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.catalog.model.CatalogEdit
import dms.application.common.OperationOutcome
import dms.application.port.storage.model.ExternalFile
import dms.domain.catalog.Catalog
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.TagName
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequest
import dms.ui.components.model.TextInputRequest
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.DocumentRow
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiState
import dms.ui.library.model.LibraryViewState
import dms.ui.library.model.YearFilter
import dms.ui.library.search.ContentSearch
import dms.ui.library.search.model.ContentSearchState
import dms.ui.library.sidebar.model.ToggleTarget
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.NameInput
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.SessionState
import dms.ui.session.model.UiMessage
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

    private val view = MutableStateFlow(value = LibraryViewState())

    val dialogs: DialogController = DialogController()

    val contentSearch: ContentSearch = ContentSearch(
        services = services,
        queries = view.map { it.query },
        scope = viewModelScope,
    )

    val state: StateFlow<LibraryUiState> = combine(
        flow = services.library.catalog,
        flow2 = session.state,
        flow3 = view,
        flow4 = contentSearch.state,
    ) { catalog, sessionState, viewState, search ->
        derive(catalog = catalog, sessionState = sessionState, view = viewState, search = search)
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = LibraryUiState())

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
    fun filterYear(year: YearFilter?) {
        view.update { it.copy(year = year) }
    }

    fun search(query: String) {
        view.update { it.copy(query = query) }
    }

    fun inspect(documentId: DocumentId?) {
        view.update { it.copy(inspected = documentId) }
    }

    fun setLayout(layout: DocumentLayout) {
        view.update { it.copy(layout = layout) }
    }

    /** Shows the documents of [selection] that match [query] and [year], as the dashboard asks. */
    fun show(selection: LibrarySelection, query: String, year: YearFilter? = null) {
        view.update {
            it.copy(selection = selection, query = query, year = year, inspected = null)
                .unfolding(selection = selection)
        }
    }

    /** Shows every document with [documentId] open in the inspector. */
    fun reveal(documentId: DocumentId) {
        view.update {
            it.copy(
                selection = LibrarySelection.Folder(path = RelativePath.ROOT),
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
        chooseAndAdd(target = RelativePath.ROOT)
    }

    fun addFiles(locations: List<String>, target: RelativePath = selectedFolder()) {
        if (locations.isEmpty()) {
            session.report(message = UiMessage(text = "Only files can be added, not folders", isError = true))
            return
        }
        session.perform(label = "Adding ${locations.size} file(s)") {
            val added = services.documents.addDocuments(
                files = locations.map { ExternalFile(location = it) },
                targetFolder = target,
            )
            val done = added.count { it.outcome is OperationOutcome.Done }
            val duplicates = added.count { it.duplicates.isNotEmpty() }
            val problem = added.firstOrNull { it.outcome !is OperationOutcome.Done }
                ?.let { "${it.source.fileName}: ${OutcomeMessages.of(outcome = it.outcome, done = "").text}" }
            val text = listOfNotNull(
                "Added $done of ${locations.size} file(s)",
                duplicates.takeIf { it > 0 }?.let { "$it already in the library under another name" },
                problem,
            ).joinToString(separator = "; ")
            session.report(message = UiMessage(text = text, isError = done < locations.size))
        }
    }

    /** Files [document] where it was dropped: moves it into a folder, or gives it a category or a tag. */
    fun drop(document: Document, target: DropTarget) {
        val name = document.path.fileName
        when (target) {
            is DropTarget.Folder -> if ((document.path.parent ?: RelativePath.ROOT) != target.path) {
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
                edit(edit = edit, done = "Filed $name under $category")
            }

            DropTarget.Uncategorized -> if (document.categoryId != null) {
                val edit = CatalogEdit.AssignCategory(documentId = document.id, categoryId = null)
                edit(edit = edit, done = "Removed the category of $name")
            }

            is DropTarget.Tag -> if (target.tag !in document.tags) {
                val edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(target.tag))
                edit(edit = edit, done = "Tagged $name with ${target.tag.value}")
            }
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

    fun renameFolder(path: RelativePath) {
        askName(title = "Rename folder", initial = path.fileName, confirm = "Rename") { name ->
            val target = NameInput.child(parent = path.parent ?: RelativePath.ROOT, raw = name)
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

    /** Creates a category; below the selected category, if one is selected. */
    fun newCategory() {
        val parentId = (view.value.selection as? LibrarySelection.Category)?.categoryId
        askText(title = "New category", label = "Name", initial = "", confirm = "Create") { name ->
            edit(edit = CatalogEdit.AddCategory(name = name.trim(), parentId = parentId))
        }
    }

    fun renameCategory(category: Category) {
        askText(title = "Rename category", label = "Name", initial = category.name, confirm = "Rename") { name ->
            edit(edit = CatalogEdit.RenameCategory(categoryId = category.id, name = name.trim()))
        }
    }

    fun removeCategory(category: Category) {
        dialogs.ask(
            request = ConfirmationRequest(
                title = "Delete category \"${category.name}\"",
                message = "The category leaves the catalog. No document uses it and no file changes.",
                affected = emptyList(),
                confirmLabel = "Delete category",
            ),
        ) {
            edit(edit = CatalogEdit.RemoveCategory(categoryId = category.id))
            view.update { it.copy(selection = LibrarySelection.Folder(path = RelativePath.ROOT)) }
        }
    }

    fun renameTag(tag: TagName) {
        askText(title = "Rename tag", label = "Tag", initial = tag.value, confirm = "Rename") { name ->
            val renamed = runCatching { TagName.of(raw = name) }.getOrNull()
            if (renamed == null) {
                session.report(message = UiMessage(text = "Not a valid tag", isError = true))
            } else {
                edit(edit = CatalogEdit.RenameTag(from = tag, to = renamed))
                view.update { it.copy(selection = LibrarySelection.Tag(tag = renamed)) }
            }
        }
    }

    fun removeTag(tag: TagName) {
        val tagged = services.library.catalog.value?.documents?.values.orEmpty().filter { tag in it.tags }
        dialogs.ask(
            request = ConfirmationRequest(
                title = "Remove tag \"${tag.value}\"",
                message = "The tag is removed from the catalog and from ${tagged.size} document(s). " +
                    "The files stay as they are.",
                affected = tagged.map { it.path.value },
                confirmLabel = "Remove tag",
            ),
        ) {
            edit(edit = CatalogEdit.RemoveTag(tag = tag))
            view.update { it.copy(selection = LibrarySelection.Folder(path = RelativePath.ROOT)) }
        }
    }

    /** Saves [edit]; reports errors always, and success only when [done] says what happened. */
    private fun edit(edit: CatalogEdit, done: String? = null) {
        session.perform(label = "Saving the catalog") {
            val outcome = services.documents.editCatalog(edit = edit)
            if (outcome !is OperationOutcome.Done || done != null) {
                session.report(message = OutcomeMessages.of(outcome = outcome, done = done.orEmpty()))
            }
        }
    }

    private fun chooseAndAdd(target: RelativePath) {
        viewModelScope.launch {
            val files = platform.chooseFiles(title = "Add files to the library")
            if (files.isNotEmpty()) {
                addFiles(locations = files, target = target)
            }
        }
    }

    /** Asks for a file or folder name and passes it on only if the file name policy allows it. */
    private fun askName(title: String, initial: String, confirm: String, submit: (String) -> Unit) {
        askText(title = title, label = "Name", initial = initial, confirm = confirm) { name ->
            val problem = NameInput.problem(raw = name)
            if (problem == null) {
                submit(name)
            } else {
                session.report(message = UiMessage(text = problem, isError = true))
            }
        }
    }

    private fun askText(title: String, label: String, initial: String, confirm: String, submit: (String) -> Unit) {
        dialogs.askText(
            request = TextInputRequest(title = title, label = label, initialValue = initial, confirmLabel = confirm),
            submit = submit,
        )
    }

    private fun selectedFolder(): RelativePath {
        return (view.value.selection as? LibrarySelection.Folder)?.path ?: RelativePath.ROOT
    }

    private fun relocated(path: RelativePath, from: RelativePath, to: RelativePath): RelativePath {
        val moves = path == from || path.isInside(folder = from)
        return if (moves) path.relocate(from = from, to = to) else path
    }

    private fun derive(
        catalog: Catalog?,
        sessionState: SessionState,
        view: LibraryViewState,
        search: ContentSearchState,
    ): LibraryUiState {
        if (catalog == null) {
            return LibraryUiState(busy = sessionState.busy, aiBusy = sessionState.aiBusy)
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
        return LibraryUiState(
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
                DocumentRow(
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
        )
    }

    private fun <T> Set<T>.flip(item: T): Set<T> {
        return if (item in this) this - item else this + item
    }

    /** The view with the folders and categories above [selection] unfolded, so the selection shows. */
    private fun LibraryViewState.unfolding(selection: LibrarySelection): LibraryViewState {
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
