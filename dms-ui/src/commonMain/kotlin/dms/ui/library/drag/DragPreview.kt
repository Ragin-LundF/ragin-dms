package dms.ui.library.drag

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.surface.IconBadge
import dms.ui.components.surface.KindBadge
import dms.ui.library.drag.model.DragItem
import dms.ui.theme.Spacing

/**
 * The card that follows the pointer while a document or category is dragged, saying what a drop would
 * do; [count] documents are dragged together when the dragged one is part of the selection.
 */
@Composable
fun DragPreview(item: DragItem, action: String?, modifier: Modifier = Modifier, count: Int = 1) {
    Surface(
        modifier = modifier.widthIn(max = 320.dp),
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = 8.dp,
        tonalElevation = 2.dp,
    ) {
        Row(
            modifier = Modifier.padding(all = Spacing.medium),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            val title = when (item) {
                is DragItem.DraggedDocument -> {
                    KindBadge(kind = item.document.kind, size = 32.dp)
                    if (count > 1) "$count documents" else item.document.metadata.title ?: item.document.path.fileName
                }

                is DragItem.DraggedCategory -> {
                    IconBadge(icon = Icons.Rounded.Category, tint = MaterialTheme.colorScheme.primary, size = 32.dp)
                    item.category.name
                }

                is DragItem.DraggedFolder -> {
                    IconBadge(icon = Icons.Rounded.Folder, tint = MaterialTheme.colorScheme.primary, size = 32.dp)
                    item.path.fileName
                }
            }
            Column {
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = action ?: if (item is DragItem.DraggedCategory) {
                        "Drop on a category, or on \"Categories\" for the top level"
                    } else {
                        "Drop on a folder"
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = if (action == null) {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    } else {
                        MaterialTheme.colorScheme.primary
                    },
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}
