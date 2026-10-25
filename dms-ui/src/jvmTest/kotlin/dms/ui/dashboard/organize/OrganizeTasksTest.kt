package dms.ui.dashboard.organize

import dms.application.document.model.DetailEdit
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.retention.model.RetentionModel
import dms.domain.rule.RuleOrigin
import dms.domain.rule.learn.model.RuleProposalModel
import dms.domain.taxonomy.model.TagNameModel
import dms.domain.task.OpenTask
import dms.ui.components.dialog.DialogController
import dms.ui.dashboard.model.TaskGroup
import dms.ui.session.model.UiMessageModel
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import java.time.LocalDate
import kotlin.io.path.listDirectoryEntries
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OrganizeTasksTest {

    private val dialogs = DialogController()

    @Test
    fun `trashing ended retention asks first, naming each file in every library folder`() {
        UiTest.run { app ->
            val ids = SampleLibrary.open(app = app, "a.pdf", "letters/b.pdf").values.map { it.id }
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.trashRetained(documentIds = ids)
            val request = checkNotNull(value = dialogs.confirmation.value)
            dialogs.dismissConfirmation()
            app.idle()

            assertEquals(expected = "Move 2 documents to the trash", actual = request.title)
            assertEquals(
                expected = setOf("Primary: a.pdf", "Sync: a.pdf", "Primary: letters/b.pdf", "Sync: letters/b.pdf"),
                actual = request.affected.toSet(),
            )
            assertEquals(expected = 2, actual = SampleLibrary.catalog(app = app).documents.size)
            assertTrue(actual = Files.exists(app.sync.resolve("letters/b.pdf")))
        }
    }

    @Test
    fun `confirmed trashing moves the files out of every folder and counts what is already gone as not trashed`() {
        UiTest.run { app ->
            val ids = SampleLibrary.open(app = app, "a.pdf", "letters/b.pdf").values.map { it.id }
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.trashRetained(documentIds = ids + DocumentIdModel(value = "gone"))
            dialogs.confirm()
            app.idle()

            assertTrue(actual = SampleLibrary.catalog(app = app).documents.isEmpty())
            assertFalse(actual = Files.exists(app.primary.resolve("a.pdf")))
            assertFalse(actual = Files.exists(app.sync.resolve("letters/b.pdf")))
            assertEquals(expected = 4, actual = app.trash.listDirectoryEntries().size)
            assertEquals(
                expected = UiMessageModel(text = "Moved 2 of 3 documents to the trash"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `marking the retention as done keeps the documents and is never flagged again`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            keepUntil(app = app, id = document.id, date = LocalDate.parse("2020-01-31"))
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.settleRetention(documentIds = listOf(document.id))
            app.idle()

            val retention = SampleLibrary.document(app = app, id = document.id).retention
            assertEquals(expected = LocalDate.parse("2020-01-31"), actual = retention?.keepUntil)
            assertNotNull(actual = retention?.settledAt)
            assertEquals(
                expected = UiMessageModel(text = "Kept 1 documents; their retention is done"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `the bulk buttons answer every retention task of the group and other groups get none`() {
        UiTest.run { app ->
            val ids = SampleLibrary.open(app = app, "a.pdf", "b.pdf").values.map { it.id }
            val organize = organizeOf(app = app, scope = backgroundScope)
            val tasks = ids.map { OpenTask.RetentionDue(documentId = it, date = LocalDate.parse("2020-01-31")) } +
                OpenTask.RestructureSuggested(folder = RelativePathModel.ROOT, fileCount = 2)

            val buttons = organize.bulkButtonsFor(group = TaskGroup.RETENTION, tasks = tasks)
            buttons.first().onClick()
            app.idle()

            assertEquals(
                expected = listOf("Mark all as done (2)", "Move all to trash (2)…"),
                actual = buttons.map { it.label },
            )
            val documents = SampleLibrary.catalog(app = app).documents.values
            assertTrue(actual = documents.all { it.retention?.settledAt != null })
            assertEquals(
                expected = emptyList(),
                actual = organize.bulkButtonsFor(group = TaskGroup.DEADLINES, tasks = tasks),
            )
        }
    }

    @Test
    fun `an accepted rule proposal becomes a learned rule for the correspondent`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            val organize = organizeOf(app = app, scope = backgroundScope)
            val energy = setOf(TagNameModel.of(raw = "energy"))

            organize.acceptRule(
                proposal = RuleProposalModel(
                    correspondent = "Sample Utility",
                    categoryId = null,
                    tags = energy,
                    documentCount = 3,
                ),
            )
            app.idle()

            val settings = SampleLibrary.catalog(app = app).settings
            val rule = settings.rules.single()
            assertEquals(expected = RuleOrigin.LEARNED, actual = rule.origin)
            assertEquals(expected = energy, actual = rule.actions.tags)
            assertEquals(expected = listOf("Sample Utility"), actual = settings.correspondents.map { it.name })
            assertEquals(
                expected = UiMessageModel(text = "Documents of Sample Utility now get the rule's values proposed"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `the calendar with the upcoming retention dates is saved in the chosen folder`() {
        UiTest.run { app ->
            val document = SampleLibrary.open(app = app, "a.pdf").getValue(key = "a.pdf")
            val date = app.services.library.clock.today().plusDays(10)
            keepUntil(app = app, id = document.id, date = date)
            val target = Files.createDirectories(app.root.resolve("calendar"))
            app.platform.nextFolder = target.toString()
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.exportCalendar()
            val message = app.session.state.first { it.message != null }.message

            val file = target.resolve("deadlines.ics")
            assertEquals(expected = UiMessageModel(text = "Saved $file"), actual = message)
            val calendar = app.read(file = file)
            assertTrue(actual = calendar.contains(other = "DTSTART;VALUE=DATE:${date.toString().replace("-", "")}"))
            assertTrue(actual = calendar.contains(other = "SUMMARY:Retention ends: a.pdf"))
        }
    }

    @Test
    fun `a calendar for a folder that does not exist is reported`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            app.platform.nextFolder = app.root.resolve("missing").toString()
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.exportCalendar()
            val message = app.session.state.first { it.message != null }.message

            assertEquals(
                expected = UiMessageModel(text = "The calendar could not be saved: file not found", isError = true),
                actual = message,
            )
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    @Test
    fun `no calendar is saved when the folder choice is cancelled`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app)
            app.platform.nextFolder = null
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.exportCalendar()
            testScheduler.runCurrent()

            assertTrue(actual = app.session.state.value.operations.isEmpty())
            assertNull(actual = app.session.state.value.message)
        }
    }

    @Test
    fun `without an open library marking the retention as done reports it`() {
        UiTest.run { app ->
            val organize = organizeOf(app = app, scope = backgroundScope)

            organize.settleRetention(documentIds = listOf(DocumentIdModel(value = "synthetic")))
            app.idle()

            assertEquals(
                expected = UiMessageModel(text = "No library is open", isError = true),
                actual = app.session.state.value.message,
            )
        }
    }

    private fun organizeOf(app: TestApp, scope: CoroutineScope): OrganizeTasks {
        return OrganizeTasks(
            services = app.services,
            session = app.session,
            dialogs = dialogs,
            platform = app.platform,
            scope = scope,
        )
    }

    private suspend fun keepUntil(app: TestApp, id: DocumentIdModel, date: LocalDate) {
        app.services.documents.editDocumentDetails(
            edit = DetailEdit.SetRetention(documentId = id, retention = RetentionModel(keepUntil = date)),
        )
    }
}
