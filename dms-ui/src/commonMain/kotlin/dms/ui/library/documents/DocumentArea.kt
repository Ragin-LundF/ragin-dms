package dms.ui.library.documents

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.SearchOff
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.PlatformActions
import dms.ui.components.layout.FileDropZone
import dms.ui.components.surface.EmptyState
import dms.ui.library.LibraryViewModel
import dms.ui.library.documents.grid.DocumentGrid
import dms.ui.library.documents.list.DocumentList
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel

/** The documents of the selection as a list or a grid; files dropped here go to the selected folder. */
@Composable
fun DocumentArea(
    state: LibraryUiStateModel,
    drag: DocumentDrag,
    platform: PlatformActions,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    val folder = (state.selection as? LibrarySelection.Folder)?.path?.value.orEmpty()
    FileDropZone(
        platform = platform,
        label = "The files go to ${folder.ifEmpty { "the top level" }} in every library folder",
        onDrop = { files -> viewModel.addFiles(locations = files) },
        modifier = modifier,
    ) {
        when {
            state.documents.isEmpty() -> EmptyState(
                icon = Icons.Rounded.SearchOff,
                title = if (state.query.isBlank()) "No documents here yet" else "Nothing matches",
                message = "Drop files here, or use Add files. They are copied into every library folder.",
                modifier = Modifier.fillMaxSize(),
            )

            state.layout == DocumentLayout.LIST -> DocumentList(
                rows = state.documents,
                inspected = state.inspected?.id,
                sort = state.sort,
                folderId = state.primaryFolderId,
                thumbnails = viewModel.thumbnails,
                drag = drag,
                onInspect = viewModel::inspect,
                onSort = viewModel::sortBy,
            )

            else -> DocumentGrid(
                rows = state.documents,
                inspected = state.inspected?.id,
                drag = drag,
                onInspect = viewModel::inspect,
            )
        }
    }
}
