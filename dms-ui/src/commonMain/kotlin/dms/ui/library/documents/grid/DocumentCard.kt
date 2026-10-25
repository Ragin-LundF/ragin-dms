package dms.ui.library.documents.grid

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.aspectRatio
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.library.documents.ClickableDocument
import dms.ui.library.documents.SelectionCircle
import dms.ui.library.documents.model.DocumentClick
import dms.ui.library.model.DocumentRowModel
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/**
 * One document of the grid: its first page at 3:4 with the amount and due date on it, then the title and
 * "sender · category". The circle on hover ticks it; while anything is ticked ([selecting]) every
 * circle shows. The document open in quick look ([inspected]) has a ring.
 */
@Composable
fun DocumentCard(
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
    val shape = MaterialTheme.shapes.large
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val document = row.document
    val title = document.metadata.title ?: document.path.fileName
    val background = when {
        selected -> colors.primaryContainer
        hovered -> colors.surfaceContainer
        else -> Color.Transparent
    }
    ClickableDocument(label = "Open $title", onClick = onClick) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape = shape)
                .background(color = background)
                .border(
                    width = if (inspected) Spacing.hairline * 2 else Spacing.hairline,
                    color = if (selected || inspected) colors.primary else Color.Transparent,
                    shape = shape,
                )
                .hoverable(interactionSource = interaction)
                .padding(all = Spacing.small),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
                PagePreview(
                    row = row,
                    folderId = folderId,
                    loader = thumbnails,
                    modifier = Modifier.fillMaxWidth().aspectRatio(ratio = PAGE_RATIO),
                )
                Caption(
                    title = title,
                    details = listOfNotNull(document.metadata.correspondent, row.categoryName)
                        .joinToString(separator = " · "),
                )
            }
            SelectionCircle(
                title = title,
                selected = selected,
                visible = hovered || selecting,
                onToggle = onToggle,
                modifier = Modifier.align(alignment = Alignment.TopStart).padding(all = Spacing.small),
            )
        }
    }
}

@Composable
private fun Caption(title: String, details: String) {
    Column {
        Text(
            text = title,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = FontWeight.Medium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = details,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

/** A page is 3 wide to 4 high. */
private const val PAGE_RATIO = 0.75f
