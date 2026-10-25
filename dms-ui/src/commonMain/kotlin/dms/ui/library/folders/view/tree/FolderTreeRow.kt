package dms.ui.library.folders.view.tree

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.library.model.RelativePathModel
import dms.ui.components.FoldToggle
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DraggableItem
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.drag.model.FolderDropPlace
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing
import kotlinx.coroutines.delay

/**
 * One row of the folder tree, indented by [depth]. A click opens the folder; [expanded] `null` means it
 * has no subfolders. Documents dropped on it move into it, and a folded row unfolds ([onExpand]) after
 * something hovered over it for a moment. [menu] shows on hover.
 */
@Composable
fun FolderTreeRow(
    path: RelativePathModel,
    label: String,
    icon: ImageVector,
    depth: Int,
    count: Int,
    selected: Boolean,
    expanded: Boolean?,
    drag: DocumentDrag,
    onOpen: () -> Unit,
    onToggle: () -> Unit,
    onExpand: () -> Unit,
    menu: (@Composable () -> Unit)? = null,
) {
    val currentOnExpand by rememberUpdatedState(newValue = onExpand)
    DropTargetArea(drag = drag, target = DropTarget.Folder(path = path, place = FolderDropPlace.TREE)) { dropHovered ->
        LaunchedEffect(dropHovered, expanded) {
            if (dropHovered && expanded == false) {
                delay(timeMillis = HOVER_OPEN_MILLIS)
                currentOnExpand()
            }
        }
        val content: @Composable () -> Unit = {
            RowContent(
                label = label,
                icon = icon,
                depth = depth,
                count = count,
                selected = selected,
                dropHovered = dropHovered,
                expanded = expanded,
                onOpen = onOpen,
                onToggle = onToggle,
                menu = menu,
            )
        }
        // The library's top level stays where it is; every other folder can be dragged into another one.
        if (path.isRoot) {
            content()
        } else {
            DraggableItem(drag = drag, item = DragItem.DraggedFolder(path = path), content = content)
        }
    }
}

@Composable
private fun RowContent(
    label: String,
    icon: ImageVector,
    depth: Int,
    count: Int,
    selected: Boolean,
    dropHovered: Boolean,
    expanded: Boolean?,
    onOpen: () -> Unit,
    onToggle: () -> Unit,
    menu: (@Composable () -> Unit)?,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.small
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val background = when {
        dropHovered -> colors.primaryContainer
        selected -> colors.surfaceContainerHigh
        hovered -> colors.surfaceContainer
        else -> Color.Transparent
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = shape)
            .background(color = background)
            .border(
                width = Spacing.hairline,
                color = if (dropHovered) colors.primary else Color.Transparent,
                shape = shape,
            )
            .hoverable(interactionSource = interaction)
            .clickable(onClick = onOpen)
            .padding(start = Spacing.xsmall + LibrarySizes.folderTreeIndent * depth, end = Spacing.xsmall)
            .padding(vertical = Spacing.xsmall),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        FoldToggle(expanded = expanded, onToggle = onToggle)
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (selected || dropHovered) colors.primary else colors.onSurfaceVariant,
            modifier = Modifier.size(size = Spacing.icon),
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = if (selected) FontWeight.Medium else FontWeight.Normal,
            color = if (selected) colors.onSurface else colors.onSurfaceVariant,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(weight = 1f),
        )
        if (count > 0) {
            Text(text = "$count", style = MaterialTheme.typography.labelSmall, color = colors.onSurfaceVariant)
        }
        menu?.let { Box(modifier = Modifier.alpha(alpha = if (hovered) 1f else 0f)) { it() } }
    }
}

private const val HOVER_OPEN_MILLIS = 600L
