package dms.ui.document

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.taxonomy.CategoryId
import dms.ui.library.model.CategoryEntry
import dms.ui.theme.Spacing

/** Chooses the category of a document, or none. */
@Composable
fun CategoryPicker(
    selected: CategoryId?,
    categories: List<CategoryEntry>,
    enabled: Boolean,
    onSelect: (CategoryId?) -> Unit,
) {
    var expanded by remember { mutableStateOf(value = false) }
    val current = categories.firstOrNull { it.category.id == selected }?.category?.name ?: "No category"
    Box {
        OutlinedButton(onClick = { expanded = true }, enabled = enabled, modifier = Modifier.fillMaxWidth()) {
            Icon(
                imageVector = Icons.Rounded.Category,
                contentDescription = null,
                modifier = Modifier.size(size = Spacing.icon),
            )
            Spacer(modifier = Modifier.width(width = Spacing.small))
            Text(
                text = current,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(weight = 1f),
            )
            Icon(imageVector = Icons.Rounded.ExpandMore, contentDescription = null)
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(
                text = { Text(text = "No category") },
                onClick = {
                    expanded = false
                    onSelect(null)
                },
            )
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
                    onClick = {
                        expanded = false
                        onSelect(entry.category.id)
                    },
                )
            }
        }
    }
}
