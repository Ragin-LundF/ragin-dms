package dms.ui.library.documents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.SearchOff
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.PlatformActions
import dms.ui.components.layout.FileDropZone
import dms.ui.components.surface.EmptyState
import dms.ui.library.LibraryViewModel
import dms.ui.library.documents.grid.DocumentGrid
import dms.ui.library.documents.list.DocumentList
import dms.ui.library.documents.tiles.CategoryTiles
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.selection.SelectionBar
import dms.ui.theme.Spacing

/**
 * The documents of the view: the category tiles on "All documents" without filters, then the grouped
 * grid or list, and the bulk bar while documents are ticked. Files dropped here go to the filtered
 * folder, or the top level.
 */
@Composable
fun DocumentArea(
    state: LibraryUiStateModel,
    drag: DocumentDrag,
    platform: PlatformActions,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    val folder = state.filters.filterIsInstance<FacetFilter.Folder>().firstOrNull()?.path?.value.orEmpty()
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        if (state.tiles.isNotEmpty()) {
            CategoryTiles(
                tiles = state.tiles,
                onPick = { viewModel.addFilter(filter = FacetFilter.Category(categoryId = it)) },
            )
        }
        Box(modifier = Modifier.weight(weight = 1f).fillMaxWidth()) {
            FileDropZone(
                platform = platform,
                label = "The files go to ${folder.ifEmpty { "the top level" }} in every library folder",
                onDrop = { files -> viewModel.files.addFiles(locations = files) },
                modifier = Modifier.fillMaxSize(),
            ) {
                Documents(state = state, drag = drag, viewModel = viewModel)
            }
            if (state.selectedDocuments.isNotEmpty()) {
                SelectionBar(
                    state = state,
                    viewModel = viewModel,
                    modifier = Modifier.align(alignment = Alignment.BottomCenter).padding(bottom = Spacing.large),
                )
            }
        }
    }
}

@Composable
private fun Documents(state: LibraryUiStateModel, drag: DocumentDrag, viewModel: LibraryViewModel) {
    if (state.documents.isEmpty()) {
        EmptyDocuments(filtered = state.filtered, onClear = viewModel::clearFilters)
        return
    }
    when (state.layout) {
        DocumentLayout.GRID -> DocumentGrid(
            groups = state.groups,
            selected = state.selectedDocuments,
            inspected = state.inspected?.id,
            folderId = state.primaryFolderId,
            thumbnails = viewModel.thumbnails,
            drag = drag,
            onClick = { id, click -> viewModel.clickDocument(documentId = id, click = click) },
            onToggle = { viewModel.documentSelection.toggle(documentId = it) },
        )

        DocumentLayout.LIST -> DocumentList(
            groups = state.groups,
            selected = state.selectedDocuments,
            inspected = state.inspected?.id,
            sort = state.sort,
            folderId = state.primaryFolderId,
            thumbnails = viewModel.thumbnails,
            drag = drag,
            onClick = { id, click -> viewModel.clickDocument(documentId = id, click = click) },
            onToggle = { viewModel.documentSelection.toggle(documentId = it) },
            onSort = viewModel::sortBy,
        )
    }
}

@Composable
private fun EmptyDocuments(filtered: Boolean, onClear: () -> Unit) {
    if (filtered) {
        EmptyState(
            icon = Icons.Rounded.SearchOff,
            title = "No documents match",
            message = "Remove a filter or change the search to see more.",
            modifier = Modifier.fillMaxSize(),
        ) {
            OutlinedButton(onClick = onClear) { Text(text = "Clear filters") }
        }
        return
    }
    EmptyState(
        icon = Icons.Rounded.SearchOff,
        title = "No documents here yet",
        message = "Drop files here, or use Add files. They are copied into every library folder.",
        modifier = Modifier.fillMaxSize(),
    )
}
