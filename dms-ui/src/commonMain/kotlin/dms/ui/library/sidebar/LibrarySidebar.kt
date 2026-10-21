package dms.ui.library.sidebar

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.model.LibraryUiState
import dms.ui.theme.Spacing

/** Folders, categories and tags; the selected entry decides which documents the list shows. */
@Composable
fun LibrarySidebar(
    state: LibraryUiState,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    Surface(modifier = modifier, color = MaterialTheme.colorScheme.surfaceContainerLowest) {
        LazyColumn(contentPadding = PaddingValues(all = Spacing.medium)) {
            folderEntries(state = state, busyReason = busyReason, drag = drag, viewModel = viewModel)
            taxonomyEntries(state = state, busyReason = busyReason, drag = drag, viewModel = viewModel)
        }
    }
}
