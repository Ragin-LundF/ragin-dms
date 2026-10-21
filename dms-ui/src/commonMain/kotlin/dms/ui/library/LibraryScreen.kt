package dms.ui.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import dms.ui.PlatformActions
import dms.ui.document.InspectorPanel
import dms.ui.document.InspectorViewModel
import dms.ui.library.documents.DocumentArea
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DragOverlay
import dms.ui.library.sidebar.LibrarySidebar
import dms.ui.theme.Spacing

/**
 * The library: sidebar, documents and inspector. Documents can be dragged onto a folder, category
 * or tag of the sidebar; files dragged in from the file manager go to the selected folder.
 */
@Composable
fun LibraryScreen(viewModel: LibraryViewModel, inspector: InspectorViewModel, platform: PlatformActions) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    val drag = remember(viewModel) { DocumentDrag(onDrop = viewModel::drop) }
    var origin by remember { mutableStateOf(value = Offset.Zero) }
    Box(modifier = Modifier.fillMaxSize().onGloballyPositioned { origin = it.positionInRoot() }) {
        Row(modifier = Modifier.fillMaxSize()) {
            LibrarySidebar(
                state = state,
                busyReason = busyReason,
                drag = drag,
                viewModel = viewModel,
                modifier = Modifier.width(width = Spacing.sidebarWidth).fillMaxHeight(),
            )
            VerticalDivider()
            Column(
                modifier = Modifier.weight(weight = 1f).fillMaxHeight().padding(all = Spacing.xlarge),
                verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
            ) {
                LibraryToolbar(
                    state = state,
                    busyReason = busyReason,
                    onSearch = viewModel::search,
                    onContentSearch = viewModel.contentSearch::enable,
                    onYear = viewModel::filterYear,
                    onLayout = viewModel::setLayout,
                    onAddFiles = { viewModel.addFiles() },
                    onNewFolder = viewModel::newFolder,
                )
                DocumentArea(
                    state = state,
                    drag = drag,
                    platform = platform,
                    viewModel = viewModel,
                    modifier = Modifier.weight(weight = 1f).fillMaxWidth(),
                )
            }
            state.inspected?.let { document ->
                VerticalDivider()
                InspectorPanel(
                    document = document,
                    categories = state.categories,
                    allTags = state.tags,
                    busyReason = busyReason,
                    aiBusyReason = state.aiBusy?.let { "Wait until \"$it\" finishes" },
                    viewModel = inspector,
                    onClose = { viewModel.inspect(documentId = null) },
                    modifier = Modifier.width(width = Spacing.inspectorWidth).fillMaxHeight(),
                )
            }
        }
        DragOverlay(drag = drag, origin = origin, categories = state.categories)
    }
    LibraryDialogs(viewModel = viewModel, inspector = inspector)
}
