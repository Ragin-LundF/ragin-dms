package dms.ui.library.folders.view

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.DriveFileMove
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.MoreHoriz
import androidx.compose.runtime.Composable
import dms.domain.library.model.RelativePathModel
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.library.folders.FolderBrowser

/** The ⋯ menu of a folder, the same on a tree row and on a tile; changes wait while [busyReason] is set. */
@Composable
fun FolderMenu(path: RelativePathModel, browser: FolderBrowser, busyReason: String?) {
    val editor = browser.editor
    ActionMenu(
        actions = listOf(
            ActionButtonModel(label = "Open", icon = Icons.Rounded.FolderOpen) { browser.open(path = path) },
            ActionButtonModel(label = "New subfolder", icon = Icons.Rounded.CreateNewFolder) {
                editor.newFolder(parent = path)
            },
            ActionButtonModel(
                label = "Rename",
                icon = Icons.Rounded.Edit,
                shortcut = "↵",
                disabledReason = busyReason,
            ) { editor.rename(path = path) },
            ActionButtonModel(
                label = "Move to…",
                icon = Icons.AutoMirrored.Rounded.DriveFileMove,
                disabledReason = busyReason,
            ) { editor.chooseTarget(path = path) },
            ActionButtonModel(
                label = "Delete…",
                icon = Icons.Rounded.Delete,
                shortcut = "⌫",
                disabledReason = busyReason,
            ) { editor.delete(path = path) },
        ),
        contentDescription = "Actions for ${path.fileName}",
        enabled = true,
        busyReason = busyReason,
        icon = Icons.Rounded.MoreHoriz,
    )
}
