package dms.ui.library.documents.tiles

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.document.DocumentKind
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.components.surface.KindStyles
import dms.ui.library.documents.model.CategoryTileModel
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The top-level categories as tiles above "All documents": a small stack of pages, the name and how
 * many documents it holds. A click filters by that category.
 */
@Composable
fun CategoryTiles(tiles: List<CategoryTileModel>, onPick: (CategoryIdModel) -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier.fillMaxWidth().horizontalScroll(state = rememberScrollState()),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        tiles.forEach { tile -> CategoryTile(tile = tile) { onPick(tile.categoryId) } }
    }
}

@Composable
private fun CategoryTile(tile: CategoryTileModel, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Surface(
        onClick = onClick,
        shape = MaterialTheme.shapes.large,
        color = colors.surface,
        border = BorderStroke(width = Spacing.hairline, color = colors.outlineVariant),
        modifier = Modifier.width(width = LibrarySizes.tileWidth),
    ) {
        Column(
            modifier = Modifier.padding(all = Spacing.medium),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            PageStack(kinds = tile.kinds)
            Column {
                Text(
                    text = tile.name,
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = if (tile.count == 1) "1 document" else "${tile.count} documents",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                )
            }
        }
    }
}

/** Up to three pages fanned out, each with a band in the colour of its kind. */
@Composable
private fun PageStack(kinds: List<DocumentKind>) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.extraSmall
    Box(modifier = Modifier.height(height = LibrarySizes.tileStackHeight)) {
        kinds.forEachIndexed { index, kind ->
            Column(
                modifier = Modifier
                    .offset(
                        x = LibrarySizes.tilePageOffset * index,
                        y = Spacing.hairline * 2 * (kinds.size - 1 - index),
                    )
                    .rotate(degrees = (index - 1) * FAN_DEGREES)
                    .size(width = LibrarySizes.tilePageWidth, height = LibrarySizes.tilePageHeight)
                    .clip(shape = shape)
                    .background(color = colors.surface)
                    .border(width = Spacing.hairline, color = colors.outlineVariant, shape = shape),
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(height = Spacing.small)
                        .background(color = KindStyles.color(kind = kind)),
                )
            }
        }
    }
}

private const val FAN_DEGREES = 4f
