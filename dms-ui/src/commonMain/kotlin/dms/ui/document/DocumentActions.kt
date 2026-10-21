package dms.ui.document

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Chat
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.DriveFileRenameOutline
import androidx.compose.material.icons.rounded.FileOpen
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.MoveDown
import androidx.compose.material.icons.rounded.NewLabel
import androidx.compose.material.icons.rounded.Summarize
import androidx.compose.runtime.Composable
import dms.domain.document.Document
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle

/**
 * Open and show the document, and let the AI summarize it or answer questions, which never wait;
 * and tag, classify, rename and move it, which wait for a busy library.
 */
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
    val unreadable = "The AI does not read this kind of file".takeIf { !document.kind.readableByAi }
    ActionButtons(
        buttons = listOf(
            ActionButton(
                label = "Summarize with AI",
                icon = Icons.Rounded.Summarize,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable,
            ) { viewModel.summary.open(document = document) },
            ActionButton(
                label = "Ask AI…",
                icon = Icons.AutoMirrored.Rounded.Chat,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable,
            ) { viewModel.chat.open(document = document) },
        ),
        enabled = true,
        busyReason = null,
    )
    ActionButtons(
        buttons = listOf(
            ActionButton(
                label = "Tag with AI",
                icon = Icons.Rounded.NewLabel,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable,
            ) { viewModel.tagWithAi(document = document) },
            ActionButton(
                label = "Classify with AI",
                icon = Icons.Rounded.AutoAwesome,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable,
            ) { viewModel.classifyWithAi(document = document) },
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
