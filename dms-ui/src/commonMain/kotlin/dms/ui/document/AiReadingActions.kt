package dms.ui.document

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Chat
import androidx.compose.material.icons.rounded.Summarize
import androidx.compose.runtime.Composable
import dms.domain.document.model.DocumentModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle

/**
 * Let the AI summarize the document or answer questions about it, which never wait for another run;
 * both are disabled with [unreadable] as the reason when the AI does not read this kind of file.
 */
@Composable
fun AiReadingActions(document: DocumentModel, unreadable: String?, viewModel: InspectorViewModel) {
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(
                label = "Summarize with AI",
                icon = Icons.Rounded.Summarize,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable,
            ) { viewModel.summary.open(document = document) },
            ActionButtonModel(
                label = "Ask AI…",
                icon = Icons.AutoMirrored.Rounded.Chat,
                style = ButtonStyle.TONAL,
                disabledReason = unreadable,
            ) { viewModel.chat.open(document = document) },
        ),
        enabled = true,
        busyReason = null,
    )
}
