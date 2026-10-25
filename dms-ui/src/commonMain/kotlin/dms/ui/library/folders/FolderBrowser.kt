package dms.ui.library.folders

import dms.domain.catalog.model.CatalogModel
import dms.domain.library.model.RelativePathModel
import dms.ui.components.dialog.DialogController
import dms.ui.library.LibraryTrees
import dms.ui.library.folders.model.FolderBrowserStateModel
import dms.ui.library.folders.model.FolderViewStateModel
import dms.ui.library.model.FolderEntryModel
import dms.ui.library.state.DiskFolders
import dms.ui.library.state.PanelWidth
import dms.ui.library.state.model.LayoutPanel
import dms.ui.library.state.model.PanelLayoutModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.SessionStateModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update

/**
 * The folder view of the library, a file manager: which folder is open, the tree beside it, searching
 * inside the open folder, and the actions on folders ([editor]) and on the tree's width ([treeLayout]).
 * Confirmations go through [dialogs].
 */
class FolderBrowser(
    services: AppServices,
    private val session: LibrarySession,
    dialogs: DialogController,
    private val disk: DiskFolders,
    scope: CoroutineScope,
) {

    private val view = MutableStateFlow(value = FolderViewStateModel())
    private val catalog = services.library.catalog

    val editor: FolderEditor = FolderEditor(
        services = services,
        session = session,
        dialogs = dialogs,
        view = view,
        documents = { catalog.value?.documents?.values.orEmpty() },
        foldersOnDisk = { disk.folders.value },
        onFoldersChanged = disk::refresh,
        scope = scope,
    )

    val treeLayout: PanelWidth = PanelWidth(
        panel = LayoutPanel.FOLDER_TREE,
        layout = session.layout,
        stored = session.state.map { it.preferences.layout }.distinctUntilChanged(),
        current = { session.state.value.preferences.layout },
        scope = scope,
    )

    val state: StateFlow<FolderBrowserStateModel> = combine(
        flow = catalog,
        flow2 = session.state,
        flow3 = view,
        flow4 = treeLayout.state,
        flow5 = disk.folders,
    ) { catalog, sessionState, viewState, tree, onDisk ->
        derive(catalog = catalog, sessionState = sessionState, view = viewState, tree = tree, onDisk = onDisk)
    }.stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = FolderBrowserStateModel())

    /** Opens [path] and unfolds the tree down to it; a name being typed is saved first. */
    fun open(path: RelativePathModel) {
        editor.saveName()
        val parents = generateSequence(seed = path.parent) { it.parent }.filterNot { it.isRoot }.toSet()
        view.update { it.copy(open = path, query = "", expanded = it.expanded + parents) }
    }

    /** Folds or unfolds [path] in the tree. */
    fun toggle(path: RelativePathModel) {
        view.update { state ->
            state.copy(expanded = if (path in state.expanded) state.expanded - path else state.expanded + path)
        }
    }

    /** Unfolds [path], as a folded row does after something hovered over it for a moment. */
    fun expand(path: RelativePathModel) {
        view.update { it.copy(expanded = it.expanded + path) }
    }

    /** Shows only the documents of the open folder whose title or file name contains [query]. */
    fun search(query: String) {
        view.update { it.copy(query = query) }
    }

    /** A new folder in the open folder, named in place. */
    fun newFolder() {
        editor.newFolder(parent = view.value.open)
    }

    /** Renames the open folder in place (↵); the top level has no name. */
    fun renameOpenFolder() {
        editor.rename(path = view.value.open)
    }

    /** Deletes the open folder after asking (⌫); the top level cannot be deleted. */
    fun deleteOpenFolder() {
        editor.delete(path = view.value.open)
    }

    /** The rows of the "Move to…" picker: every folder, also the empty ones on disk. */
    fun pickerRows(expanded: Set<RelativePathModel>): List<FolderEntryModel> {
        val documents = catalog.value?.documents?.values.orEmpty()
        return LibraryTrees.folders(documents = documents, foldersOnDisk = disk.folders.value, expanded = expanded)
            .drop(n = 1)
    }

    private fun derive(
        catalog: CatalogModel?,
        sessionState: SessionStateModel,
        view: FolderViewStateModel,
        tree: PanelLayoutModel,
        onDisk: Set<RelativePathModel>,
    ): FolderBrowserStateModel {
        val documents = catalog?.documents?.values.orEmpty()
        val folders = FolderContents.knownFolders(documents = documents, foldersOnDisk = onDisk)
        return FolderBrowserStateModel(
            open = view.open,
            crumbs = FolderContents.crumbs(open = view.open),
            tree = LibraryTrees.folders(documents = documents, foldersOnDisk = onDisk, expanded = view.expanded)
                .drop(n = 1),
            libraryCount = documents.size,
            tiles = FolderContents.tiles(open = view.open, folders = folders, documents = documents),
            documents = FolderContents.documentsIn(open = view.open, documents = documents, query = view.query),
            naming = view.naming,
            query = view.query,
            searchPlaceholder = FolderTexts.searchPlaceholder(open = view.open),
            treeWidth = tree.width,
            treeHidden = tree.folded,
            busyReason = sessionState.busy?.let { OutcomeMessages.waitFor(operation = it) },
        )
    }
}
