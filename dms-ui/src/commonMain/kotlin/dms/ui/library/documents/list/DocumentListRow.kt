package dms.ui.library.documents.list

import androidx.compose.foundation.background
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
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.field.FieldValues
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.components.format.DateTexts
import dms.ui.components.surface.LabelChip
import dms.ui.library.DocumentColumns
import dms.ui.library.documents.ClickableDocument
import dms.ui.library.documents.SelectionCircle
import dms.ui.library.documents.model.DocumentClick
import dms.ui.library.model.DocumentRowModel
import dms.ui.preview.ThumbnailLoader

/**
 * One document as a row: the circle that ticks it, page thumbnail, title and file, category, tags,
 * sender, date and amount.
 */
@Composable
fun DocumentListRow(
    row: DocumentRowModel,
    selected: Boolean,
    inspected: Boolean,
    selecting: Boolean,
    folderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    onClick: (DocumentClick) -> Unit,
    onToggle: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val document = row.document
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val background = when {
        selected || inspected -> colors.primaryContainer
        hovered -> colors.surfaceContainer
        else -> Color.Transparent
    }
    val title = document.metadata.title ?: document.path.fileName
    ClickableDocument(label = "Open $title", onClick = onClick) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape = MaterialTheme.shapes.medium)
                .background(color = background)
                .hoverable(interactionSource = interaction)
                .padding(horizontal = DocumentColumns.padding, vertical = DocumentColumns.rowPadding),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = DocumentColumns.gap),
        ) {
            SelectionCircle(title = title, selected = selected, visible = hovered || selecting, onToggle = onToggle)
            RowThumbnail(document = document, folderId = folderId, loader = thumbnails)
            NameCell(document = document, modifier = Modifier.weight(weight = DocumentColumns.NAME))
            Box(modifier = Modifier.weight(weight = DocumentColumns.CATEGORY)) {
                val name = row.categoryName
                if (name == null) {
                    CellText(text = "none", color = colors.onSurfaceVariant, modifier = Modifier, italic = true)
                } else {
                    LabelChip(text = name, color = colors.tertiary)
                }
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
                text = row.amount?.let { FieldValues.format(value = it) }.orEmpty(),
                color = colors.onSurface,
                modifier = Modifier.weight(weight = DocumentColumns.AMOUNT),
                textAlign = TextAlign.End,
            )
        }
    }
}

@Composable
private fun CellText(
    text: String,
    color: Color,
    modifier: Modifier,
    textAlign: TextAlign = TextAlign.Start,
    italic: Boolean = false,
) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = color,
        fontStyle = if (italic) FontStyle.Italic else FontStyle.Normal,
        textAlign = textAlign,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier,
    )
}
