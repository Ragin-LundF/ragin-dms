package dms.ui.document.edit

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.library.model.CategoryEntryModel
import dms.ui.theme.Spacing

/** Chooses the category of a document, or none; subcategories are indented under their parent. */
@Composable
fun CategoryMenu(
    expanded: Boolean,
    categories: List<CategoryEntryModel>,
    onSelect: (CategoryIdModel?) -> Unit,
    onDismiss: () -> Unit,
) {
    DropdownMenu(expanded = expanded, onDismissRequest = onDismiss) {
        DropdownMenuItem(text = { Text(text = "No category") }, onClick = { onSelect(null) })
        categories.forEach { entry ->
            DropdownMenuItem(
                text = {
                    Text(
                        text = entry.category.name,
                        modifier = Modifier.padding(start = Spacing.large * entry.depth),
                        maxLines = 1,
                        softWrap = false,
                        overflow = TextOverflow.Ellipsis,
                    )
                },
                onClick = { onSelect(entry.category.id) },
            )
        }
    }
}
