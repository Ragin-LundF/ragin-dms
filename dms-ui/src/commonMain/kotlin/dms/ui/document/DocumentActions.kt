package dms.ui.document

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.DriveFileRenameOutline
import androidx.compose.material.icons.rounded.FileOpen
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.MoveDown
import androidx.compose.material.icons.rounded.NewLabel
import androidx.compose.runtime.Composable
import dms.domain.document.Document
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle

/**
 * Open and show the document, and let the AI summarize it or answer questions ([AiReadingActions]);
 * tag and classify it, which wait for another AI run ([aiBusyReason]); and rename and move it,
 * which wait for a busy library ([busyReason]).
 */
@Composable
fun DocumentActions(document: Document, busyReason: String?, aiBusyReason: String?, viewModel: InspectorViewModel) {
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
    AiReadingActions(document = document, unreadable = unreadable, viewModel = viewModel)
    ActionButtons(
        buttons = listOf(
            ActionButton(
                label = "Tag with AI",
                icon = Icons.Rounded.NewLabel,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable ?: aiBusyReason,
            ) { viewModel.tagWithAi(document = document) },
            ActionButton(
                label = "Classify with AI",
                icon = Icons.Rounded.AutoAwesome,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable ?: aiBusyReason,
            ) { viewModel.classifyWithAi(document = document) },
            ActionButton(
                label = "Rename…",
                icon = Icons.Rounded.DriveFileRenameOutline,
                style = ButtonStyle.TEXT,
                disabledReason = busyReason,
            ) { viewModel.rename(document = document) },
            ActionButton(
                label = "Move…",
                icon = Icons.Rounded.MoveDown,
                style = ButtonStyle.TEXT,
                disabledReason = busyReason,
            ) { viewModel.moveToFolder(document = document) },
        ),
        enabled = true,
        busyReason = null,
    )
}
