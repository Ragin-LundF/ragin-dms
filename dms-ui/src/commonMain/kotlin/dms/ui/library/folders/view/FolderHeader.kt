package dms.ui.library.folders.view

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.library.model.RelativePathModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.drag.model.FolderDropPlace
import dms.ui.library.folders.FolderBrowser
import dms.ui.library.folders.model.FolderBrowserStateModel
import dms.ui.library.folders.model.FolderCrumbModel
import dms.ui.library.folders.view.tree.TreeToggleButton
import dms.ui.theme.Spacing

/**
 * The path of the open folder, each segment opening its folder and taking drops, and the actions on the
 * open folder; the button that shows a hidden tree comes first.
 */
@Composable
fun FolderHeader(
    state: FolderBrowserStateModel,
    browser: FolderBrowser,
    drag: DocumentDrag,
    onAddFiles: (RelativePathModel) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        if (state.treeHidden) {
            TreeToggleButton(hidden = true, onClick = browser.treeLayout::toggleFolded)
        }
        Row(modifier = Modifier.weight(weight = 1f), verticalAlignment = Alignment.CenterVertically) {
            state.crumbs.forEachIndexed { index, crumb ->
                if (index > 0) {
                    Text(
                        text = "/",
                        color = MaterialTheme.colorScheme.outline,
                        style = MaterialTheme.typography.titleMedium,
                    )
                }
                Crumb(crumb = crumb, last = index == state.crumbs.lastIndex, drag = drag) {
                    browser.open(path = crumb.path)
                }
            }
        }
        ActionButtons(
            buttons = actions(state = state, browser = browser, onAddFiles = onAddFiles),
            enabled = state.busyReason == null,
            busyReason = state.busyReason,
        )
    }
}

@Composable
private fun Crumb(crumb: FolderCrumbModel, last: Boolean, drag: DocumentDrag, onOpen: () -> Unit) {
    val target = DropTarget.Folder(path = crumb.path, place = FolderDropPlace.BREADCRUMB)
    DropTargetArea(drag = drag, target = target) { hovered ->
        val colors = MaterialTheme.colorScheme
        val shape = MaterialTheme.shapes.small
        Text(
            text = crumb.name,
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.SemiBold,
            color = if (last) colors.onSurface else colors.onSurfaceVariant,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier
                .clip(shape = shape)
                .background(color = if (hovered) colors.primaryContainer else Color.Transparent)
                .border(
                    width = Spacing.hairline,
                    color = if (hovered) colors.primary else Color.Transparent,
                    shape = shape,
                )
                .clickable(onClick = onOpen)
                .padding(horizontal = Spacing.small, vertical = Spacing.xsmall),
        )
    }
}

private fun actions(
    state: FolderBrowserStateModel,
    browser: FolderBrowser,
    onAddFiles: (RelativePathModel) -> Unit,
): List<ActionButtonModel> {
    val newFolder = ActionButtonModel(label = "New folder", icon = Icons.Rounded.CreateNewFolder) {
        browser.newFolder()
    }
    val addFiles = ActionButtonModel(
        label = "Add files here",
        icon = Icons.Rounded.Add,
        style = ButtonStyle.FILLED,
    ) { onAddFiles(state.open) }
    if (state.open.isRoot) {
        return listOf(newFolder, addFiles)
    }
    val rename = ActionButtonModel(label = "Rename", icon = Icons.Rounded.Edit) { browser.renameOpenFolder() }
    val delete = ActionButtonModel(label = "Delete", icon = Icons.Rounded.Delete) { browser.deleteOpenFolder() }
    return listOf(newFolder, rename, delete, addFiles)
}
