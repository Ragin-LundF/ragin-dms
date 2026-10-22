package dms.ui.library.sidebar.item

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.runtime.Composable
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableItem
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.CategoryEntryModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.sidebar.model.ToggleTarget
import dms.ui.library.sidebar.model.TreeToggleModel

/**
 * One category; documents dropped on it get it, and a category dropped on it moves inside. It can be
 * dragged itself. Its 3-dot menu adds a subcategory, renames or deletes it.
 */
@Composable
fun CategoryItem(
    entry: CategoryEntryModel,
    selection: LibrarySelection,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val category = entry.category
    val actions = listOf(
        ActionButtonModel(label = "Add Subcategory", icon = Icons.Rounded.Add) {
            viewModel.newSubcategory(parent = category)
        },
        ActionButtonModel(label = "Rename", icon = Icons.Rounded.Edit) {
            viewModel.renameCategory(category = category)
        },
        ActionButtonModel(
            label = "Delete",
            icon = Icons.Rounded.Delete,
            disabledReason = "Documents still use this category".takeIf { entry.documentCount > 0 },
        ) { viewModel.removeCategory(category = category) },
    )
    DropTargetArea(drag = drag, target = DropTarget.Category(categoryId = category.id)) { hovered ->
        DraggableItem(drag = drag, item = DragItem.DraggedCategory(category = category)) {
            SidebarItem(
                label = category.name,
                icon = Icons.Rounded.Category,
                count = entry.documentCount,
                depth = entry.depth,
                selected = selection == LibrarySelection.Category(categoryId = category.id),
                dropHovered = hovered,
                toggle = TreeToggleModel(
                    expanded = entry.expanded.takeIf { entry.hasChildren },
                    onToggle = { viewModel.toggle(target = ToggleTarget.Category(categoryId = category.id)) },
                ),
                menu = {
                    ActionMenu(
                        actions = actions,
                        contentDescription = "Actions for ${category.name}",
                        enabled = busyReason == null,
                        busyReason = busyReason,
                    )
                },
            ) { viewModel.select(selection = LibrarySelection.Category(categoryId = category.id)) }
        }
    }
}
