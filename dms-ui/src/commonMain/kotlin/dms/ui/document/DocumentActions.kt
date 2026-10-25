package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Chat
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.DriveFileRenameOutline
import androidx.compose.material.icons.rounded.FileOpen
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.MoveDown
import androidx.compose.material.icons.rounded.NewLabel
import androidx.compose.material.icons.rounded.Summarize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import dms.domain.document.model.DocumentModel
import dms.ui.components.ActionMenu
import dms.ui.components.StyledButton
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.theme.Spacing

/**
 * "Open", then icons to show the file and let the AI answer questions or summarize it, which never wait
 * for another run, and a menu to tag and classify it, which wait for another AI run ([aiBusyReason]),
 * and to rename, move or trash it, which wait for a busy library ([busyReason]).
 */
@Composable
fun DocumentActions(
    document: DocumentModel,
    busyReason: String?,
    aiBusyReason: String?,
    viewModel: InspectorViewModel,
) {
    val unreadable = "The AI does not read this kind of file".takeIf { !document.kind.readableByAi }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
    ) {
        StyledButton(
            button = ActionButtonModel(label = "Open", icon = Icons.Rounded.FileOpen, style = ButtonStyle.FILLED) {
                viewModel.open(document = document)
            },
            enabled = true,
        )
        ActionIconButton(
            button = ActionButtonModel(
                label = "Show in ${viewModel.fileManagerName}",
                icon = Icons.Rounded.FolderOpen,
            ) { viewModel.showInFileManager(document = document) },
        )
        ActionIconButton(
            button = ActionButtonModel(
                label = "Ask AI…",
                icon = Icons.AutoMirrored.Rounded.Chat,
                disabledReason = unreadable,
            ) { viewModel.chat.open(document = document) },
        )
        ActionIconButton(
            button = ActionButtonModel(
                label = "Summarize with AI",
                icon = Icons.Rounded.Summarize,
                disabledReason = unreadable,
            ) { viewModel.summary.open(document = document) },
        )
        ActionMenu(
            actions = moreActions(
                document = document,
                aiReason = unreadable ?: aiBusyReason,
                busyReason = busyReason,
                viewModel = viewModel,
            ),
            contentDescription = "More actions for ${document.path.fileName}",
            enabled = true,
            busyReason = null,
        )
    }
}

private fun moreActions(
    document: DocumentModel,
    aiReason: String?,
    busyReason: String?,
    viewModel: InspectorViewModel,
): List<ActionButtonModel> {
    return listOf(
        ActionButtonModel(label = "Tag with AI", icon = Icons.Rounded.NewLabel, disabledReason = aiReason) {
            viewModel.tagWithAi(document = document)
        },
        ActionButtonModel(label = "Classify with AI", icon = Icons.Rounded.AutoAwesome, disabledReason = aiReason) {
            viewModel.classifyWithAi(document = document)
        },
        ActionButtonModel(
            label = "Rename…",
            icon = Icons.Rounded.DriveFileRenameOutline,
            disabledReason = busyReason,
        ) { viewModel.rename(document = document) },
        ActionButtonModel(label = "Move…", icon = Icons.Rounded.MoveDown, disabledReason = busyReason) {
            viewModel.moveToFolder(document = document)
        },
        ActionButtonModel(label = "Move to trash…", icon = Icons.Rounded.Delete, disabledReason = busyReason) {
            viewModel.trash(document = document)
        },
    )
}
