package dms.ui.dashboard.section

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.DocumentScanner
import androidx.compose.material.icons.rounded.Event
import androidx.compose.material.icons.rounded.MoveToInbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.domain.document.model.DocumentIdModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.organize.model.UpcomingRowModel
import dms.ui.theme.Spacing

/**
 * The inbox while it is switched on (how many documents wait, and what to do with them) and the next
 * deadline and retention dates.
 */
@Composable
fun OrganizeSection(
    state: DashboardUiStateModel,
    onOpenInbox: () -> Unit,
    onAnalyzeInbox: () -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onExportCalendar: () -> Unit,
    scans: List<DocumentIdModel>?,
    onFindScans: () -> Unit,
    onMakeSearchable: () -> Unit,
) {
    InboxCard(state = state, onOpenInbox = onOpenInbox, onAnalyzeInbox = onAnalyzeInbox)
    ScansCard(
        scans = scans,
        busyReason = state.busy?.let { "Wait until \"$it\" finishes" },
        onFind = onFindScans,
        onMake = onMakeSearchable,
    )
    UpcomingCard(rows = state.upcoming, onOpenDocument = onOpenDocument, onExportCalendar = onExportCalendar)
}

@Composable
private fun InboxCard(state: DashboardUiStateModel, onOpenInbox: () -> Unit, onAnalyzeInbox: () -> Unit) {
    val inbox = state.inbox ?: return
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Inbox (${inbox.count})",
            icon = Icons.Rounded.MoveToInbox,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = "New documents in ${inbox.path} of your library folders",
        )
        if (inbox.count == 0) {
            Text(text = "Nothing waits in the inbox.", style = MaterialTheme.typography.bodyMedium)
        }
        val aiBusyReason = state.aiBusy?.let { "Wait until \"$it\" finishes" }
        val analyze = ActionButtonModel(
            label = "Analyze all",
            icon = Icons.Rounded.AutoAwesome,
            style = ButtonStyle.TONAL,
            disabledReason = aiBusyReason ?: "The inbox is empty".takeIf { inbox.count == 0 },
            onClick = onAnalyzeInbox,
        )
        val open = ActionButtonModel(
            label = "Open inbox",
            icon = Icons.AutoMirrored.Rounded.ArrowForward,
            onClick = onOpenInbox,
        )
        ActionButtons(
            buttons = if (inbox.aiAnalysis) listOf(analyze, open) else listOf(open),
            enabled = true,
            busyReason = null,
        )
    }
}

@Composable
private fun UpcomingCard(
    rows: List<UpcomingRowModel>,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onExportCalendar: () -> Unit,
) {
    if (rows.isEmpty()) {
        return
    }
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Upcoming",
            icon = Icons.Rounded.Event,
            tint = MaterialTheme.colorScheme.tertiary,
            subtitle = "The next deadlines and retention dates",
            trailing = {
                ActionButtons(
                    buttons = listOf(
                        ActionButtonModel(label = "Export .ics", style = ButtonStyle.TEXT, onClick = onExportCalendar),
                    ),
                    enabled = true,
                    busyReason = null,
                )
            },
        )
        rows.forEach { row ->
            Row(
                modifier = Modifier.fillMaxWidth().clickable { onOpenDocument(row.documentId) }.padding(
                    vertical = Spacing.xsmall,
                ),
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
            ) {
                Text(text = row.date, style = MaterialTheme.typography.labelLarge)
                Text(text = row.what, style = MaterialTheme.typography.bodyMedium, maxLines = 1, softWrap = false)
                Text(
                    text = row.document,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun ScansCard(scans: List<DocumentIdModel>?, busyReason: String?, onFind: () -> Unit, onMake: () -> Unit) {
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = scans?.let { "${it.size} scans without text" } ?: "Scans without text",
            icon = Icons.Rounded.DocumentScanner,
            tint = MaterialTheme.colorScheme.secondary,
            subtitle = "PDFs search cannot read, until they get a text layer",
        )
        val buttons = if (scans == null) {
            listOf(ActionButtonModel(label = "Find scans", onClick = onFind))
        } else {
            listOf(
                ActionButtonModel(
                    label = "Make ${scans.size} searchable",
                    style = ButtonStyle.TONAL,
                    disabledReason = "No scan without text".takeIf { scans.isEmpty() },
                    onClick = onMake,
                ),
                ActionButtonModel(label = "Look again", style = ButtonStyle.TEXT, onClick = onFind),
            )
        }
        ActionButtons(buttons = buttons, enabled = busyReason == null, busyReason = busyReason)
    }
}
