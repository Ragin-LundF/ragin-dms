package dms.ui.library

import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.unit.dp
import dms.ui.PlatformActions
import dms.ui.components.layout.ResizeHandle
import dms.ui.document.QuickLookViewModel
import dms.ui.document.quicklook.QuickLookSheet
import dms.ui.library.documents.DocumentArea
import dms.ui.library.documents.grid.DocumentGrid
import dms.ui.library.documents.model.DocumentGroupModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DragOverlay
import dms.ui.library.filter.FilterBar
import dms.ui.library.folders.view.FolderView
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.LibrarySidebar
import dms.ui.library.state.DocumentRows
import dms.ui.session.OutcomeMessages
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The library: the sidebar of views (foldable to an icon rail, resizable), and on the right the search
 * with the facet chips above the documents, or the folder view while "Folders" is selected. Quick look
 * opens over the documents on the right. Keys outside the text fields follow the concept's table: "/"
 * search, Space quick look, ↑/↓ next document, ⌘A / Ctrl+A select all, Esc closes quick look and then
 * clears the ticks; in the folder view ↵ renames and ⌫ deletes the open folder. ⌘\ is the menu's.
 */
@Composable
fun LibraryScreen(viewModel: LibraryViewModel, quickLookViewModel: QuickLookViewModel, platform: PlatformActions) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
    val drag = remember(viewModel) { DocumentDrag(onDrop = viewModel::drop) }
    var origin by remember { mutableStateOf(value = Offset.Zero) }
    val keys = remember { FocusRequester() }
    LaunchedEffect(key1 = keys) { keys.requestFocus() }
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(color = MaterialTheme.colorScheme.background)
            .onGloballyPositioned { origin = it.positionInRoot() }
            .focusRequester(focusRequester = keys)
            .focusable()
            .onKeyEvent { event -> LibraryKeys.handle(event = event, state = state, viewModel = viewModel) },
    ) {
        Row(modifier = Modifier.fillMaxSize()) {
            SidebarPane(state = state, busyReason = busyReason, viewModel = viewModel)
            Box(
                modifier = Modifier
                    .weight(weight = 1f)
                    .fillMaxHeight()
                    .pointerInput(keys) {
                        // A press anywhere in the documents takes the keys back from the search field.
                        awaitEachGesture {
                            awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                            keys.requestFocus()
                        }
                    },
            ) {
                if (state.selection == LibrarySelection.Folders) {
                    FolderPane(viewModel = viewModel, drag = drag, platform = platform)
                } else {
                    DocumentsPane(
                        state = state,
                        busyReason = busyReason,
                        drag = drag,
                        platform = platform,
                        viewModel = viewModel,
                    )
                }
                QuickLookSheet(
                    categories = state.categories,
                    allTags = state.tags,
                    folderId = state.primaryFolderId,
                    busyReason = busyReason,
                    aiBusyReason = state.aiBusy?.let { OutcomeMessages.waitFor(operation = it) },
                    viewModel = quickLookViewModel,
                    modifier = Modifier.align(alignment = Alignment.CenterEnd),
                )
            }
        }
        DragOverlay(drag = drag, origin = origin, categories = state.categories, selected = state.selectedDocuments)
    }
    LibraryDialogs(viewModel = viewModel, quickLookViewModel = quickLookViewModel)
}

/** The sidebar at its stored width, or folded into the rail, with the edge that resizes it. */
@Composable
private fun SidebarPane(state: LibraryUiStateModel, busyReason: String?, viewModel: LibraryViewModel) {
    val sidebar by viewModel.sidebarWidth.state.collectAsState()
    LibrarySidebar(
        state = state,
        folded = sidebar.folded,
        busyReason = busyReason,
        viewModel = viewModel,
        modifier = Modifier
            .width(width = if (sidebar.folded) LibrarySizes.rail else sidebar.width.dp)
            .fillMaxHeight(),
    )
    if (sidebar.folded) {
        VerticalDivider()
        return
    }
    ResizeHandle(
        onResize = viewModel.sidebarWidth::resize,
        onResizeEnd = viewModel.sidebarWidth::endResize,
        onDoubleClick = viewModel.sidebarWidth::resetWidth,
    )
}

/** The folder view, its documents drawn as the library's cards, which open in quick look in folder order. */
@Composable
private fun FolderPane(viewModel: LibraryViewModel, drag: DocumentDrag, platform: PlatformActions) {
    val state by viewModel.state.collectAsState()
    val catalog by viewModel.catalog.collectAsState()
    val searchFocusRequests by viewModel.searchFocusRequests.collectAsState()
    FolderView(
        browser = viewModel.folders,
        drag = drag,
        platform = platform,
        onAddFiles = viewModel.files::chooseAndAdd,
        onDropFiles = { locations, target -> viewModel.files.addFiles(locations = locations, target = target) },
        searchFocusRequests = searchFocusRequests,
        modifier = Modifier.fillMaxSize(),
    ) { documents ->
        val order = documents.map { it.id }
        DocumentGrid(
            groups = listOf(
                DocumentGroupModel(
                    title = "",
                    rows = catalog?.let { open -> documents.map { DocumentRows.rowOf(document = it, catalog = open) } }
                        .orEmpty(),
                ),
            ),
            selected = state.selectedDocuments,
            inspected = state.inspected?.id,
            folderId = state.primaryFolderId,
            thumbnails = viewModel.thumbnails,
            drag = drag,
            onClick = { id, _ -> viewModel.quickLook.open(documentId = id, order = order) },
            onToggle = viewModel.documentSelection::toggle,
            headers = false,
        )
    }
}

@Composable
private fun DocumentsPane(
    state: LibraryUiStateModel,
    busyReason: String?,
    drag: DocumentDrag,
    platform: PlatformActions,
    viewModel: LibraryViewModel,
) {
    Column(
        modifier = Modifier.fillMaxSize().padding(horizontal = Spacing.xlarge, vertical = Spacing.large),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        FilterBar(state = state, busyReason = busyReason, viewModel = viewModel, modifier = Modifier.fillMaxWidth())
        DocumentArea(
            state = state,
            drag = drag,
            platform = platform,
            viewModel = viewModel,
            modifier = Modifier.weight(weight = 1f).fillMaxWidth(),
        )
    }
}
