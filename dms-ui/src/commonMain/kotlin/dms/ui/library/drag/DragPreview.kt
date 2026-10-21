package dms.ui.library.drag

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.domain.document.Document
import dms.ui.components.surface.KindBadge
import dms.ui.theme.Spacing

/** The card that follows the pointer while a document is dragged, saying what a drop would do. */
@Composable
fun DragPreview(document: Document, action: String?, modifier: Modifier = Modifier) {
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
            KindBadge(kind = document.kind, size = 32.dp)
            Column {
                Text(
                    text = document.metadata.title ?: document.path.fileName,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = action ?: "Drop on a folder, category or tag",
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
