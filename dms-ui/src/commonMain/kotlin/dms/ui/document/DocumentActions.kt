package dms.ui.document

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.DriveFileRenameOutline
import androidx.compose.material.icons.rounded.FileOpen
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.MoveDown
import androidx.compose.runtime.Composable
import dms.domain.document.Document
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle

/** Open and show the document, which never waits, and rename and move it, which wait for a busy library. */
@Composable
fun DocumentActions(document: Document, busyReason: String?, viewModel: InspectorViewModel) {
    ActionButtons(
        buttons = listOf(
            ActionButton(label = "Open", icon = Icons.Rounded.FileOpen, style = ButtonStyle.FILLED) {
                viewModel.open(document = document)
            },
            ActionButton(
                label = "Show in ${viewModel.fileManagerName}",
                icon = Icons.Rounded.FolderOpen,
                style = ButtonStyle.TONAL,
            ) { viewModel.showInFileManager(document = document) },
        ),
        enabled = true,
        busyReason = null,
    )
    ActionButtons(
        buttons = listOf(
            ActionButton(label = "Rename…", icon = Icons.Rounded.DriveFileRenameOutline, style = ButtonStyle.TEXT) {
                viewModel.rename(document = document)
            },
            ActionButton(label = "Move…", icon = Icons.Rounded.MoveDown, style = ButtonStyle.TEXT) {
                viewModel.moveToFolder(document = document)
            },
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
