package dms.ui.library.documents

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.FileSizes
import dms.ui.components.surface.KindBadge
import dms.ui.components.surface.LabelChip
import dms.ui.library.model.DocumentRowModel
import dms.ui.theme.Spacing

/** One document as a card of the grid: a large kind badge, the title, the folder, category and date. */
@Composable
fun DocumentCard(row: DocumentRowModel, selected: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val document = row.document
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().height(height = 188.dp),
        shape = MaterialTheme.shapes.large,
        color = if (selected) colors.primaryContainer else colors.surface,
        border = BorderStroke(
            width = if (selected) 2.dp else Spacing.hairline,
            color = if (selected) colors.primary else colors.outlineVariant,
        ),
        shadowElevation = 1.dp,
    ) {
        Column(
            modifier = Modifier.padding(all = Spacing.large),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                KindBadge(kind = document.kind, size = 44.dp)
                Spacer(modifier = Modifier.weight(weight = 1f))
                Text(
                    text = FileSizes.format(bytes = document.size),
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.onSurfaceVariant,
                )
            }
            Text(
                text = document.metadata.title ?: document.path.fileName,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = document.path.parent?.value?.ifEmpty { "Top level" } ?: "Top level",
                style = MaterialTheme.typography.bodySmall,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Spacer(modifier = Modifier.weight(weight = 1f))
            Row(verticalAlignment = Alignment.CenterVertically) {
                row.categoryName?.let { name ->
                    LabelChip(
                        text = name,
                        color = colors.tertiary,
                        modifier = Modifier.weight(weight = 1f, fill = false),
                    )
                }
                Spacer(modifier = Modifier.weight(weight = 1f))
                Text(
                    text = document.metadata.documentDate?.toString().orEmpty(),
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.onSurfaceVariant,
                )
            }
        }
    }
}
