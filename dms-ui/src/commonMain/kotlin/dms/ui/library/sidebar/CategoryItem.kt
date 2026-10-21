package dms.ui.library.sidebar

import androidx.compose.runtime.Composable
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.CategoryEntry
import dms.ui.library.model.LibrarySelection

/** One category of the sidebar; while selected it offers to rename or delete it. */
@Composable
fun CategoryItem(
    entry: CategoryEntry,
    selection: LibrarySelection,
    busyReason: String?,
    viewModel: LibraryViewModel,
) {
    val category = entry.category
    val selected = selection == LibrarySelection.Category(categoryId = category.id)
    SidebarItem(label = category.name, count = entry.documentCount, depth = entry.depth, selected = selected) {
        viewModel.select(selection = LibrarySelection.Category(categoryId = category.id))
    }
    if (selected) {
        ActionButtons(
            buttons = listOf(
                ActionButton(label = "Rename…") { viewModel.renameCategory(category = category) },
                ActionButton(
                    label = "Delete",
                    disabledReason = "Documents still use this category".takeIf { entry.documentCount > 0 },
                ) { viewModel.removeCategory(category = category) },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
