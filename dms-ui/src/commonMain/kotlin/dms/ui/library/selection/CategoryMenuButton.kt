package dms.ui.library.selection

import androidx.compose.foundation.layout.Box
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.HorizontalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import dms.ui.library.model.LibraryUiStateModel

/** "Category" of the bulk bar: every category, subcategories indented, and "Without a category". */
@Composable
fun CategoryMenuButton(state: LibraryUiStateModel, disabledReason: String?, bulk: BulkActions) {
    var open by remember { mutableStateOf(value = false) }
    Box {
        BarButton(label = "Category", icon = Icons.Rounded.Category, disabledReason = disabledReason) { open = true }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            state.categories.forEach { entry ->
                MenuChoice(text = entry.category.name, depth = entry.depth) {
                    open = false
                    bulk.assignCategory(categoryId = entry.category.id)
                }
            }
            if (state.categories.isNotEmpty()) {
                HorizontalDivider()
            }
            MenuChoice(text = "Without a category", depth = 0, italic = true) {
                open = false
                bulk.assignCategory(categoryId = null)
            }
        }
    }
}
