package dms.ui.document.quicklook

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Chat
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.DriveFileRenameOutline
import androidx.compose.material.icons.rounded.FileOpen
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.MoreHoriz
import androidx.compose.material.icons.rounded.MoveDown
import androidx.compose.material.icons.rounded.NewLabel
import androidx.compose.material.icons.rounded.Summarize
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentModel
import dms.ui.components.ActionMenu
import dms.ui.components.StyledButton
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.document.ActionIconButton
import dms.ui.document.QuickLookViewModel
import dms.ui.theme.Spacing

/**
 * "Open", then icons to show the file and to ask the AI about it, which never wait for another run,
 * and ⋯ to summarize, tag and classify it, which wait for another AI run ([aiBusyReason]), and to
 * rename, move or trash it, which wait for a busy library ([busyReason]). Then the key hint and Close.
 */
@Composable
fun QuickLookTopBar(
    document: DocumentModel,
    busyReason: String?,
    aiBusyReason: String?,
    viewModel: QuickLookViewModel,
    onClose: () -> Unit,
) {
    val unreadable = "The AI does not read this kind of file".takeIf { !document.kind.readableByAi }
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = Spacing.medium, vertical = Spacing.small),
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
                label = "Ask about this document…",
                icon = Icons.AutoMirrored.Rounded.Chat,
                disabledReason = unreadable,
            ) { viewModel.chat.open(document = document) },
        )
        ActionMenu(
            actions = moreActions(
                document = document,
                unreadable = unreadable,
                aiReason = unreadable ?: aiBusyReason,
                busyReason = busyReason,
                viewModel = viewModel,
            ),
            contentDescription = "Rename, move, trash ${document.path.fileName}…",
            enabled = true,
            busyReason = null,
            icon = Icons.Rounded.MoreHoriz,
        )
        Text(
            text = "↑↓ next",
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.outline,
            maxLines = 1,
            softWrap = false,
            modifier = Modifier.weight(weight = 1f).padding(start = Spacing.small),
        )
        IconButton(onClick = onClose) {
            Icon(imageVector = Icons.Rounded.Close, contentDescription = "Close quick look")
        }
    }
}

private fun moreActions(
    document: DocumentModel,
    unreadable: String?,
    aiReason: String?,
    busyReason: String?,
    viewModel: QuickLookViewModel,
): List<ActionButtonModel> {
    return listOf(
        ActionButtonModel(label = "Summarize with AI", icon = Icons.Rounded.Summarize, disabledReason = unreadable) {
            viewModel.summary.open(document = document)
        },
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
