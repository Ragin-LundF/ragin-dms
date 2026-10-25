package dms.ui.dashboard.attention

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.DocumentScanner
import androidx.compose.material.icons.rounded.Event
import androidx.compose.material.icons.rounded.MoveToInbox
import dms.domain.document.model.DocumentIdModel
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.dashboard.attention.model.AttentionListModel
import dms.ui.dashboard.attention.model.AttentionRowModel
import dms.ui.dashboard.attention.model.AttentionSeverity
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.model.TaskFileModel
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.organize.model.InboxSummaryModel
import dms.ui.dashboard.organize.model.UpcomingRowModel
import dms.ui.session.OutcomeMessages

/**
 * "Needs your attention": the inbox, the next dates, the open tasks and the scans without text in
 * one list, most urgent first, cut to [LIMIT] rows; the tasks screen shows every task.
 */
object AttentionRows {

    const val LIMIT = 5

    /** [scans] are the PDFs without a text layer, or `null` while nobody looked for them. */
    fun of(
        state: DashboardUiStateModel,
        scans: List<DocumentIdModel>?,
        handler: AttentionHandler,
    ): AttentionListModel {
        val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
        val all = listOfNotNull(state.inbox?.let { inboxRow(inbox = it, state = state, handler = handler) }) +
            state.upcoming.map { upcomingRow(row = it, state = state, handler = handler) } +
            taskRows(state = state, busyReason = busyReason) +
            listOfNotNull(scansRow(scans = scans, busyReason = busyReason, handler = handler))
        return AttentionListModel(
            rows = all.sortedBy { it.severity.ordinal }.take(n = LIMIT),
            total = all.size,
            headerButtons = headerButtons(state = state, scans = scans, busyReason = busyReason, handler = handler),
        )
    }

    /** How urgent an open task of [group] is: dates that passed first, then what needs a decision. */
    fun severityOf(group: TaskGroup): AttentionSeverity {
        return when (group) {
            TaskGroup.DEADLINES,
            TaskGroup.RETENTION,
            -> AttentionSeverity.PASSED

            TaskGroup.MISSING,
            TaskGroup.MODIFIED,
            TaskGroup.MOVED,
            TaskGroup.DUPLICATES,
            TaskGroup.CONFLICTS,
            TaskGroup.CATALOG_MERGE,
            TaskGroup.FAILED_OPERATIONS,
            -> AttentionSeverity.SOON

            TaskGroup.SUGGESTIONS,
            TaskGroup.LEARNED_RULES,
            TaskGroup.NEW_FILES,
            TaskGroup.RESTRUCTURE,
            -> AttentionSeverity.INFO
        }
    }

    private fun inboxRow(
        inbox: InboxSummaryModel,
        state: DashboardUiStateModel,
        handler: AttentionHandler,
    ): AttentionRowModel? {
        if (inbox.count == 0) {
            return null
        }
        val analyze = ActionButtonModel(
            label = "Analyze all",
            icon = Icons.Rounded.AutoAwesome,
            style = ButtonStyle.TONAL,
            disabledReason = state.aiBusy?.let { OutcomeMessages.waitFor(operation = it) },
            onClick = handler::analyzeInbox,
        )
        val open = ActionButtonModel(
            label = "Open inbox",
            icon = Icons.AutoMirrored.Rounded.ArrowForward,
            onClick = handler::openInbox,
        )
        return AttentionRowModel(
            key = "inbox",
            severity = AttentionSeverity.INFO,
            title = if (inbox.count == 1) {
                "1 new document in the inbox"
            } else {
                "${inbox.count} new documents in the inbox"
            },
            detail = "In ${inbox.path} of your library folders",
            icon = Icons.Rounded.MoveToInbox,
            buttons = listOfNotNull(analyze.takeIf { inbox.aiAnalysis }),
            opens = listOf(open),
        )
    }

    private fun upcomingRow(
        row: UpcomingRowModel,
        state: DashboardUiStateModel,
        handler: AttentionHandler,
    ): AttentionRowModel {
        val folderId = state.primaryFolderId
        val path = row.path
        return AttentionRowModel(
            key = "upcoming:${row.documentId.value}:${row.what}:${row.date}",
            severity = AttentionSeverity.SOON,
            title = row.document,
            detail = "${row.what} on ${row.date}",
            icon = Icons.Rounded.Event,
            preview = if (folderId != null && path != null) {
                TaskFileModel(label = "Open", folderId = folderId, path = path)
            } else {
                null
            },
            opens = listOf(ActionButtonModel(label = "Open") { handler.openDocument(documentId = row.documentId) }),
        )
    }

    private fun taskRows(state: DashboardUiStateModel, busyReason: String?): List<AttentionRowModel> {
        return state.groups.flatMap { group ->
            group.rows.map { row ->
                AttentionRowModel(
                    key = "task:${row.key.value}",
                    severity = severityOf(group = group.group),
                    title = row.title,
                    detail = row.detail,
                    icon = group.group.icon,
                    preview = row.preview,
                    buttons = row.buttons.map { it.waitingFor(busyReason = busyReason) },
                    opens = row.opens,
                    copies = row.copies,
                )
            }
        }
    }

    private fun scansRow(
        scans: List<DocumentIdModel>?,
        busyReason: String?,
        handler: AttentionHandler,
    ): AttentionRowModel? {
        if (scans.isNullOrEmpty()) {
            return null
        }
        val make = ActionButtonModel(
            label = "Make ${scans.size} searchable",
            style = ButtonStyle.TONAL,
            onClick = handler::makeScansSearchable,
        )
        val lookAgain = ActionButtonModel(label = "Look again", style = ButtonStyle.TEXT, onClick = handler::findScans)
        return AttentionRowModel(
            key = "scans",
            severity = AttentionSeverity.INFO,
            title = if (scans.size == 1) "1 scan without text" else "${scans.size} scans without text",
            detail = "PDFs search cannot read, until they get a text layer",
            icon = Icons.Rounded.DocumentScanner,
            buttons = listOf(make, lookAgain).map { it.waitingFor(busyReason = busyReason) },
        )
    }

    /** Looking for scans and saving the calendar belong to no single row, so they sit in the header. */
    private fun headerButtons(
        state: DashboardUiStateModel,
        scans: List<DocumentIdModel>?,
        busyReason: String?,
        handler: AttentionHandler,
    ): List<ActionButtonModel> {
        val findScans = ActionButtonModel(
            label = "Find scans without text",
            style = ButtonStyle.TEXT,
            onClick = handler::findScans,
        ).waitingFor(busyReason = busyReason)
        val exportCalendar = ActionButtonModel(
            label = "Export .ics",
            style = ButtonStyle.TEXT,
            onClick = handler::exportCalendar,
        )
        return listOfNotNull(
            findScans.takeIf { scans.isNullOrEmpty() },
            exportCalendar.takeIf { state.upcoming.isNotEmpty() },
        )
    }

    /** A button that changes the library waits while another library operation runs. */
    private fun ActionButtonModel.waitingFor(busyReason: String?): ActionButtonModel {
        return if (busyReason == null || disabledReason != null) this else copy(disabledReason = busyReason)
    }
}
