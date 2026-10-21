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
import dms.ui.library.sidebar.item.CategoryItem
import dms.ui.library.sidebar.item.SidebarItem
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.sidebar.model.ToggleTarget

/** The category section of the sidebar: "Uncategorized" and the category tree, which folds. */
fun LazyListScope.categoryEntries(
    state: LibraryUiState,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val selection = state.selection
    val expanded = SidebarGroup.CATEGORIES !in state.collapsedGroups
    item(key = "categories") {
        SidebarSection(
            title = "Categories",
            expanded = expanded,
            onToggle = { viewModel.toggle(target = ToggleTarget.Group(group = SidebarGroup.CATEGORIES)) },
            addLabel = "New category",
            busyReason = busyReason,
        ) { viewModel.newCategory() }
    }
    if (!expanded) {
        return
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
}
