package dms.ui.library

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.ui.components.DialogHost
import dms.ui.document.InspectorPanel
import dms.ui.document.InspectorViewModel
import dms.ui.library.sidebar.LibrarySidebar
import dms.ui.theme.Spacing

/** The library: sidebar, document list and inspector. */
@Composable
fun LibraryScreen(viewModel: LibraryViewModel, inspector: InspectorViewModel) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    Column(modifier = Modifier.fillMaxSize().padding(all = Spacing.large)) {
        LibraryToolbar(
            query = state.query,
            busyReason = busyReason,
            onSearch = viewModel::search,
            onAddFiles = { viewModel.addFiles() },
            onNewFolder = viewModel::newFolder,
        )
        HorizontalDivider()
        Row(modifier = Modifier.weight(weight = 1f).fillMaxWidth()) {
            LibrarySidebar(
                state = state,
                busyReason = busyReason,
                viewModel = viewModel,
                modifier = Modifier.width(width = Spacing.sidebarWidth).fillMaxHeight(),
            )
            VerticalDivider()
            DocumentList(
                rows = state.documents,
                inspected = state.inspected?.id,
                onInspect = viewModel::inspect,
                modifier = Modifier.weight(weight = 1f).fillMaxHeight(),
            )
            VerticalDivider()
            InspectorPanel(
                document = state.inspected,
                categories = state.categories,
                allTags = state.tags,
                busyReason = busyReason,
                viewModel = inspector,
                modifier = Modifier.width(width = Spacing.inspectorWidth).fillMaxHeight(),
            )
        }
    }
    DialogHost(controller = viewModel.dialogs)
    DialogHost(controller = inspector.dialogs)
}
