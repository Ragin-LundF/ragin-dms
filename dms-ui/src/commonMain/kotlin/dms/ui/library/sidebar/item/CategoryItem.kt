package dms.ui.library.sidebar.item

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.CategoryEntry
import dms.ui.library.model.LibrarySelection
import dms.ui.library.sidebar.model.ToggleTarget
import dms.ui.library.sidebar.model.TreeToggle
import dms.ui.theme.Spacing

/** One category; documents dropped on it get it. While selected it offers to rename or delete it. */
@Composable
fun CategoryItem(
    entry: CategoryEntry,
    selection: LibrarySelection,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val category = entry.category
    val selected = selection == LibrarySelection.Category(categoryId = category.id)
    DropTargetArea(drag = drag, target = DropTarget.Category(categoryId = category.id)) { hovered ->
        SidebarItem(
            label = category.name,
            icon = Icons.Rounded.Category,
            count = entry.documentCount,
            depth = entry.depth,
            selected = selected,
            dropHovered = hovered,
            toggle = TreeToggle(
                expanded = entry.expanded.takeIf { entry.hasChildren },
                onToggle = { viewModel.toggle(target = ToggleTarget.Category(categoryId = category.id)) },
            ),
        ) { viewModel.select(selection = LibrarySelection.Category(categoryId = category.id)) }
    }
    if (selected) {
        ActionButtons(
            buttons = listOf(
                ActionButton(label = "Rename", icon = Icons.Rounded.Edit, style = ButtonStyle.TEXT) {
                    viewModel.renameCategory(category = category)
                },
                ActionButton(
                    label = "Delete",
                    icon = Icons.Rounded.Delete,
                    style = ButtonStyle.TEXT,
                    disabledReason = "Documents still use this category".takeIf { entry.documentCount > 0 },
                ) { viewModel.removeCategory(category = category) },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
            modifier = Modifier.padding(start = Spacing.large),
        )
    }
}
