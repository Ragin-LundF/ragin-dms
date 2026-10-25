package dms.ui.dashboard.attention

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.task.model.TaskKeyModel
import dms.ui.components.model.ActionButtonModel
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.attention.model.AttentionSeverity
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.model.TaskFileModel
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskGroupStateModel
import dms.ui.dashboard.model.TaskRowStateModel
import dms.ui.dashboard.organize.model.InboxSummaryModel
import dms.ui.dashboard.organize.model.UpcomingRowModel
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class AttentionRowsTest {

    private val handler = RecordingHandler()

    @Test
    fun `nothing to do gives no rows and only the button to look for scans`() {
        val attention = AttentionRows.of(state = DashboardUiStateModel(), scans = null, handler = handler)

        assertEquals(expected = 0, actual = attention.total)
        assertEquals(expected = emptyList(), actual = attention.rows)
        assertEquals(expected = listOf("Find scans without text"), actual = attention.headerButtons.map { it.label })
    }

    @Test
    fun `at most five rows are shown, the total counts all of them`() {
        val state = DashboardUiStateModel(groups = listOf(group(group = TaskGroup.NEW_FILES, count = 7)))

        val attention = AttentionRows.of(state = state, scans = null, handler = handler)

        assertEquals(expected = AttentionRows.LIMIT, actual = attention.rows.size)
        assertEquals(expected = 7, actual = attention.total)
    }

    @Test
    fun `passed dates come first, then decisions due soon, then the rest`() {
        val state = DashboardUiStateModel(
            groups = listOf(
                group(group = TaskGroup.SUGGESTIONS, count = 1),
                group(group = TaskGroup.CONFLICTS, count = 1),
                group(group = TaskGroup.RETENTION, count = 1),
            ),
            inbox = InboxSummaryModel(path = "Inbox", count = 2, aiAnalysis = false),
            upcoming = listOf(upcoming()),
        )

        val scans = listOf(DocumentIdModel(value = "scan"))

        val attention = AttentionRows.of(state = state, scans = scans, handler = handler)

        assertEquals(
            expected = listOf(
                "RETENTION 0" to AttentionSeverity.PASSED,
                "Sample bill" to AttentionSeverity.SOON,
                "CONFLICTS 0" to AttentionSeverity.SOON,
                "2 new documents in the inbox" to AttentionSeverity.INFO,
                "SUGGESTIONS 0" to AttentionSeverity.INFO,
            ),
            actual = attention.rows.map { it.title to it.severity },
        )
        assertEquals(expected = 6, actual = attention.total)
    }

    @Test
    fun `every kind of task has a severity, and only deadlines and retention passed a date`() {
        val passed = TaskGroup.entries.filter { AttentionRows.severityOf(group = it) == AttentionSeverity.PASSED }

        assertEquals(expected = listOf(TaskGroup.DEADLINES, TaskGroup.RETENTION), actual = passed)
        assertEquals(expected = AttentionSeverity.INFO, actual = AttentionRows.severityOf(group = TaskGroup.NEW_FILES))
        assertEquals(expected = AttentionSeverity.SOON, actual = AttentionRows.severityOf(group = TaskGroup.DUPLICATES))
    }

    @Test
    fun `while the library is busy a task waits and says why, opening its file still works`() {
        val state = DashboardUiStateModel(
            groups = listOf(group(group = TaskGroup.NEW_FILES, count = 1)),
            busy = "Scanning",
        )

        val row = AttentionRows.of(state = state, scans = null, handler = handler).rows.single()

        assertEquals(
            expected = listOf("Wait until \"Scanning\" finishes"),
            actual = row.buttons.map { it.disabledReason },
        )
        assertEquals(expected = listOf(null), actual = row.opens.map { it.disabledReason })
    }

    @Test
    fun `the inbox row offers the analysis only when it is switched on and opens the inbox`() {
        val plain = DashboardUiStateModel(inbox = InboxSummaryModel(path = "Inbox", count = 1, aiAnalysis = false))
        val analyzed = plain.copy(inbox = InboxSummaryModel(path = "Inbox", count = 1, aiAnalysis = true))
        val empty = plain.copy(inbox = InboxSummaryModel(path = "Inbox", count = 0, aiAnalysis = true))

        val plainRow = AttentionRows.of(state = plain, scans = null, handler = handler).rows.single()
        val analyzedRow = AttentionRows.of(state = analyzed, scans = null, handler = handler).rows.single()
        plainRow.opens.single().onClick()

        assertEquals(expected = "1 new document in the inbox", actual = plainRow.title)
        assertEquals(expected = emptyList(), actual = plainRow.buttons)
        assertEquals(expected = listOf("Analyze all"), actual = analyzedRow.buttons.map { it.label })
        assertEquals(expected = 0, actual = AttentionRows.of(state = empty, scans = null, handler = handler).total)
        assertEquals(expected = listOf("open inbox"), actual = handler.calls)
    }

    @Test
    fun `an upcoming date shows its document from the primary folder and opens it in the library`() {
        val primary = LibraryFolderIdModel(value = "primary")
        val state = DashboardUiStateModel(primaryFolderId = primary, upcoming = listOf(upcoming()))

        val attention = AttentionRows.of(state = state, scans = null, handler = handler)
        val row = attention.rows.single()
        row.opens.single().onClick()

        assertEquals(expected = "Due date on 2026-11-05", actual = row.detail)
        assertEquals(
            expected = TaskFileModel(
                label = "Open",
                folderId = primary,
                path = RelativePathModel.of(raw = "bills/a.pdf"),
            ),
            actual = row.preview,
        )
        assertEquals(expected = listOf("open document"), actual = handler.calls)
        assertTrue(actual = attention.headerButtons.any { it.label == "Export .ics" })
    }

    @Test
    fun `found scans get a row to make them searchable and the header stops offering the search`() {
        val scans = listOf(DocumentIdModel(value = "a"), DocumentIdModel(value = "b"))

        val attention = AttentionRows.of(state = DashboardUiStateModel(), scans = scans, handler = handler)
        val row = attention.rows.single()
        row.buttons.first().onClick()

        assertEquals(expected = "2 scans without text", actual = row.title)
        assertEquals(expected = listOf("Make 2 searchable", "Look again"), actual = row.buttons.map { it.label })
        assertEquals(expected = emptyList(), actual = attention.headerButtons)
        assertEquals(expected = listOf("make searchable"), actual = handler.calls)
        assertEquals(
            expected = 0,
            actual = AttentionRows.of(state = DashboardUiStateModel(), scans = emptyList(), handler = handler).total,
        )
    }

    @Test
    fun `a new file found by the scan needs attention with the task's own buttons`() {
        UiTest.run { app ->
            app.write(file = app.sync.resolve("stray.pdf"), content = "stray")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val state = viewModel.state.first { state -> state.groups.any { it.group == TaskGroup.NEW_FILES } }
            val row = AttentionRows.of(state = state, scans = null, handler = handler).rows.single()

            assertEquals(expected = AttentionSeverity.INFO, actual = row.severity)
            assertEquals(expected = listOf("Adopt", "Move to trash…", "Dismiss"), actual = row.buttons.map { it.label })
            assertEquals(expected = listOf("Open"), actual = row.opens.map { it.label })
        }
    }

    private fun group(group: TaskGroup, count: Int): TaskGroupStateModel {
        return TaskGroupStateModel(
            group = group,
            rows = List(size = count) { index ->
                TaskRowStateModel(
                    key = TaskKeyModel(value = "${group.name}-$index"),
                    title = "${group.name} $index",
                    detail = "",
                    buttons = listOf(ActionButtonModel(label = "Act") {}),
                    opens = listOf(ActionButtonModel(label = "Open") {}),
                    preview = null,
                )
            },
        )
    }

    private fun upcoming(): UpcomingRowModel {
        return UpcomingRowModel(
            documentId = DocumentIdModel(value = "bill"),
            date = "2026-11-05",
            what = "Due date",
            document = "Sample bill",
            path = RelativePathModel.of(raw = "bills/a.pdf"),
        )
    }

    private class RecordingHandler : AttentionHandler {

        val calls = mutableListOf<String>()

        override fun openInbox() {
            calls += "open inbox"
        }

        override fun analyzeInbox() {
            calls += "analyze inbox"
        }

        override fun openDocument(documentId: DocumentIdModel) {
            calls += "open document"
        }

        override fun findScans() {
            calls += "find scans"
        }

        override fun makeScansSearchable() {
            calls += "make searchable"
        }

        override fun exportCalendar() {
            calls += "export calendar"
        }
    }
}
