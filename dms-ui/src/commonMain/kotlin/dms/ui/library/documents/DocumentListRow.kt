package dms.ui.library.documents

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import dms.ui.components.FileSizes
import dms.ui.components.surface.KindBadge
import dms.ui.components.surface.LabelChip
import dms.ui.library.DocumentColumns
import dms.ui.library.model.DocumentRow

/** One document as a row: kind, title and file, category, tags, date and size. */
@Composable
fun DocumentListRow(row: DocumentRow, selected: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val document = row.document
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val background = when {
        selected -> colors.primaryContainer
        hovered -> colors.surfaceContainer
        else -> Color.Transparent
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = MaterialTheme.shapes.medium)
            .background(color = background)
            .hoverable(interactionSource = interaction)
            .clickable(onClick = onClick)
            .padding(horizontal = DocumentColumns.padding, vertical = DocumentColumns.rowPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = DocumentColumns.gap),
    ) {
        KindBadge(kind = document.kind)
        NameCell(document = document, modifier = Modifier.weight(weight = DocumentColumns.NAME))
        Box(modifier = Modifier.weight(weight = DocumentColumns.CATEGORY)) {
            row.categoryName?.let { LabelChip(text = it, color = colors.tertiary) }
        }
        TagChips(tags = document.tags.sorted(), modifier = Modifier.weight(weight = DocumentColumns.TAGS))
        Text(
            text = document.metadata.documentDate?.toString().orEmpty(),
            style = MaterialTheme.typography.bodySmall,
            modifier = Modifier.weight(weight = DocumentColumns.DATE),
            maxLines = 1,
        )
        Text(
            text = FileSizes.format(bytes = document.size),
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
            modifier = Modifier.weight(weight = DocumentColumns.SIZE),
            maxLines = 1,
        )
    }
}
