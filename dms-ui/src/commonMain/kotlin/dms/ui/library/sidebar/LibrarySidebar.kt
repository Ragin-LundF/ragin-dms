package dms.ui.library.sidebar

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.History
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.item.SidebarItem
import dms.ui.theme.Spacing

/**
 * Recently added, smart folders, folders, categories and tags; the selected entry decides which
 * documents the list shows.
 */
@Composable
fun LibrarySidebar(
    state: LibraryUiStateModel,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    Surface(modifier = modifier, color = MaterialTheme.colorScheme.surfaceContainerLowest) {
        LazyColumn(contentPadding = PaddingValues(all = Spacing.medium)) {
            item(key = "recent") {
                SidebarItem(
                    label = "Recently added",
                    icon = Icons.Rounded.History,
                    count = null,
                    depth = 0,
                    selected = state.selection == LibrarySelection.Recent,
                    dropHovered = false,
                ) { viewModel.select(selection = LibrarySelection.Recent) }
            }
            smartFolderEntries(state = state, busyReason = busyReason, viewModel = viewModel)
            folderEntries(state = state, busyReason = busyReason, drag = drag, viewModel = viewModel)
            categoryEntries(state = state, busyReason = busyReason, drag = drag, viewModel = viewModel)
            tagEntries(state = state, busyReason = busyReason, drag = drag, viewModel = viewModel)
        }
    }
}
