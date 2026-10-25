package dms.ui.library.documents.list

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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.components.format.DateTexts
import dms.ui.components.format.FileSizes
import dms.ui.components.surface.LabelChip
import dms.ui.library.DocumentColumns
import dms.ui.library.model.DocumentRowModel
import dms.ui.preview.ThumbnailLoader

/** One document as a row: page thumbnail, title and file, category, tags, correspondent, date and size. */
@Composable
fun DocumentListRow(
    row: DocumentRowModel,
    selected: Boolean,
    folderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    onClick: () -> Unit,
) {
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
        RowThumbnail(document = document, folderId = folderId, loader = thumbnails)
        NameCell(document = document, modifier = Modifier.weight(weight = DocumentColumns.NAME))
        Box(modifier = Modifier.weight(weight = DocumentColumns.CATEGORY)) {
            row.categoryName?.let { LabelChip(text = it, color = colors.tertiary) }
        }
        TagChips(tags = document.tags.sorted(), modifier = Modifier.weight(weight = DocumentColumns.TAGS))
        CellText(
            text = document.metadata.correspondent.orEmpty(),
            color = colors.onSurfaceVariant,
            modifier = Modifier.weight(weight = DocumentColumns.CORRESPONDENT),
        )
        CellText(
            text = document.metadata.documentDate?.let { DateTexts.absolute(date = it) }.orEmpty(),
            color = colors.onSurface,
            modifier = Modifier.weight(weight = DocumentColumns.DATE),
        )
        CellText(
            text = FileSizes.format(bytes = document.size),
            color = colors.onSurfaceVariant,
            modifier = Modifier.weight(weight = DocumentColumns.SIZE),
            textAlign = TextAlign.End,
        )
    }
}

@Composable
private fun CellText(text: String, color: Color, modifier: Modifier, textAlign: TextAlign = TextAlign.Start) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = color,
        textAlign = textAlign,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier,
    )
}
