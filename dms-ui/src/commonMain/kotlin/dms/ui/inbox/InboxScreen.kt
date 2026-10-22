package dms.ui.inbox

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.MoveToInbox
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.ui.components.ActionButtons
import dms.ui.components.layout.ScreenHeader
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.EmptyState
import dms.ui.inbox.model.InboxUiStateModel
import dms.ui.review.ReviewDialog
import dms.ui.theme.Spacing

/**
 * The inbox: every document dropped into the inbox folder of any library folder, to analyze with the
 * AI or to move by hand. [onOpenSettings] leads to the switch while the inbox is off.
 */
@Composable
fun InboxScreen(
    viewModel: InboxViewModel,
    onOpenSettings: () -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    val aiBusyReason = state.aiBusy?.let { "Wait until \"$it\" finishes" }
    Column(
        modifier = Modifier.fillMaxSize().padding(all = Spacing.xxlarge),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        val subtitle = if (state.enabled) {
            "${state.rows.size} documents in ${state.path} of your library folders"
        } else {
            "Switched off"
        }
        ScreenHeader(title = "Inbox", subtitle = subtitle) {
            if (state.enabled && state.aiAnalysis) {
                ActionButtons(
                    buttons = listOf(
                        ActionButtonModel(
                            label = "Analyze all",
                            icon = Icons.Rounded.AutoAwesome,
                            style = ButtonStyle.FILLED,
                            disabledReason = aiBusyReason ?: "The inbox is empty".takeIf { state.rows.isEmpty() },
                            onClick = viewModel::analyzeAll,
                        ),
                    ),
                    enabled = true,
                    busyReason = null,
                )
            }
        }
        InboxBody(
            state = state,
            viewModel = viewModel,
            busyReason = busyReason,
            aiBusyReason = aiBusyReason,
            onOpenSettings = onOpenSettings,
        )
    }
    state.pick?.let { pick ->
        FolderPickerDialog(
            pick = pick,
            folders = state.folders,
            recent = state.recentTargets,
            onChange = viewModel::updatePick,
            onConfirm = viewModel::confirmMove,
            onDismiss = viewModel::cancelMove,
        )
    }
    ReviewDialog(controller = viewModel.review, onOpenDocument = onOpenDocument)
}

@Composable
private fun InboxBody(
    state: InboxUiStateModel,
    viewModel: InboxViewModel,
    busyReason: String?,
    aiBusyReason: String?,
    onOpenSettings: () -> Unit,
) {
    when {
        !state.enabled -> EmptyState(
            icon = Icons.Rounded.MoveToInbox,
            title = "The inbox is off",
            message = "Switch it on in Settings: a folder in your library, such as Inbox, where you drop new " +
                "documents on any device. Each one is then analyzed by the AI or filed by hand.",
            modifier = Modifier.fillMaxWidth(),
        ) {
            ActionButtons(
                buttons = listOf(
                    ActionButtonModel(label = "Open settings", icon = Icons.Rounded.Settings, onClick = onOpenSettings),
                ),
                enabled = true,
                busyReason = null,
            )
        }

        state.rows.isEmpty() -> EmptyState(
            icon = Icons.Rounded.MoveToInbox,
            title = "Nothing in the inbox",
            message = "Drop files into ${state.path} in any library folder. They show up here after the next scan.",
            modifier = Modifier.fillMaxWidth(),
        )

        else -> LazyColumn(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            items(items = state.rows, key = { it.documentId.value }) { row ->
                InboxRowView(
                    row = row,
                    aiAnalysis = state.aiAnalysis,
                    busyReason = busyReason,
                    aiBusyReason = aiBusyReason,
                    onAnalyze = { viewModel.analyze(documentId = row.documentId) },
                    onReview = { row.suggestion?.let { viewModel.review(suggestion = it) } },
                    onMove = { viewModel.startMove(documentId = row.documentId) },
                )
            }
        }
    }
}
