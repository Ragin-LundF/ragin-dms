package dms.ui.library.folders.view

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.surface.IconBadge
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableItem
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.drag.model.FolderDropPlace
import dms.ui.library.folders.FolderBrowser
import dms.ui.library.folders.FolderTexts
import dms.ui.library.folders.model.FolderBrowserStateModel
import dms.ui.library.folders.model.FolderNaming
import dms.ui.library.folders.model.FolderTileModel
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The "Folders" section: a tile per subfolder of the open folder, which opens it and takes drops, and
 * the tile of a new folder while its name is typed. Shows nothing while there is neither.
 */
@Composable
fun FolderTiles(state: FolderBrowserStateModel, browser: FolderBrowser, drag: DocumentDrag) {
    val adding = state.naming as? FolderNaming.New
    if (state.tiles.isEmpty() && adding == null) {
        return
    }
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        FolderSectionTitle(title = "Folders", count = state.tiles.size)
        LazyVerticalGrid(
            columns = GridCells.Adaptive(minSize = LibrarySizes.folderTileMinWidth),
            modifier = Modifier.fillMaxWidth().heightIn(max = LibrarySizes.folderTilesMaxHeight),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            if (adding != null) {
                item(key = "new-folder") {
                    TileFrame(dropHovered = false) {
                        Column(modifier = Modifier.weight(weight = 1f)) {
                            NameField(naming = adding, initial = adding.proposed, browser = browser)
                        }
                    }
                }
            }
            items(items = state.tiles, key = { it.path.value }) { tile ->
                FolderTile(tile = tile, state = state, browser = browser, drag = drag)
            }
        }
    }
}

@Composable
private fun FolderTile(
    tile: FolderTileModel,
    state: FolderBrowserStateModel,
    browser: FolderBrowser,
    drag: DocumentDrag,
) {
    val renaming = (state.naming as? FolderNaming.Rename)?.takeIf { it.path == tile.path }
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val target = DropTarget.Folder(path = tile.path, place = FolderDropPlace.TILE)
    DropTargetArea(drag = drag, target = target) { dropHovered ->
        DraggableItem(drag = drag, item = DragItem.DraggedFolder(path = tile.path)) {
            TileContent(
                tile = tile,
                state = state,
                browser = browser,
                renaming = renaming,
                dropHovered = dropHovered,
                interaction = interaction,
                hovered = hovered,
            )
        }
    }
}

@Composable
private fun TileContent(
    tile: FolderTileModel,
    state: FolderBrowserStateModel,
    browser: FolderBrowser,
    renaming: FolderNaming.Rename?,
    dropHovered: Boolean,
    interaction: MutableInteractionSource,
    hovered: Boolean,
) {
    TileFrame(
        dropHovered = dropHovered,
        modifier = Modifier.hoverable(interactionSource = interaction).clickable { browser.open(path = tile.path) },
    ) {
        Column(modifier = Modifier.weight(weight = 1f)) {
            if (renaming != null) {
                NameField(naming = renaming, initial = tile.name, browser = browser)
            } else {
                Text(
                    text = tile.name,
                    style = MaterialTheme.typography.bodyMedium,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Text(
                text = FolderTexts.tileCaption(tile = tile),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
        }
        Box(modifier = Modifier.alpha(alpha = if (hovered) 1f else 0f)) {
            FolderMenu(path = tile.path, browser = browser, busyReason = state.busyReason)
        }
    }
}

@Composable
private fun TileFrame(
    dropHovered: Boolean,
    modifier: Modifier = Modifier,
    content: @Composable RowScope.() -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.medium
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = shape)
            .background(color = if (dropHovered) colors.primaryContainer else colors.surface)
            .border(
                width = Spacing.hairline,
                color = if (dropHovered) colors.primary else colors.outlineVariant,
                shape = shape,
            )
            .then(other = modifier)
            .padding(horizontal = Spacing.medium, vertical = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        IconBadge(icon = Icons.Rounded.Folder, tint = colors.primary, size = 32.dp)
        content()
    }
}

@Composable
private fun NameField(naming: FolderNaming, initial: String, browser: FolderBrowser) {
    FolderNameField(
        initial = initial,
        onType = browser.editor::type,
        onSave = { browser.editor.commitName(naming = naming, text = it) },
        onCancel = browser.editor::cancelName,
    )
}
