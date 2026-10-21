package dms.ui.library.sidebar

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Inbox
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiState

/** The category and tag sections of the sidebar, each entry a drop target. */
fun LazyListScope.taxonomyEntries(
    state: LibraryUiState,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val selection = state.selection
    item(key = "categories") {
        SidebarSection(title = "Categories", addLabel = "New category", busyReason = busyReason) {
            viewModel.newCategory()
        }
    }
    item(key = "uncategorized") {
        DropTargetArea(drag = drag, target = DropTarget.Uncategorized) { hovered ->
            SidebarItem(
                label = "Uncategorized",
                icon = Icons.Rounded.Inbox,
                count = state.uncategorizedCount,
                depth = 0,
                selected = selection == LibrarySelection.Uncategorized,
                dropHovered = hovered,
            ) { viewModel.select(selection = LibrarySelection.Uncategorized) }
        }
    }
    items(items = state.categories, key = { "category:${it.category.id.value}" }) { entry ->
        CategoryItem(entry = entry, selection = selection, busyReason = busyReason, drag = drag, viewModel = viewModel)
    }
    item(key = "tags") { SidebarSection(title = "Tags") }
    items(items = state.tags, key = { "tag:${it.value}" }) { tag ->
        TagItem(
            tag = tag,
            count = state.tagCounts[tag] ?: 0,
            selection = selection,
            busyReason = busyReason,
            drag = drag,
            viewModel = viewModel,
        )
    }
}
