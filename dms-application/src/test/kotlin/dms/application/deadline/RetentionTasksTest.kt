package dms.application.deadline

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.document.model.DetailEdit
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.retention.model.RetentionModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.task.OpenTask
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class RetentionTasksTest {

    @Test
    fun `no document is ever flagged unless the user set a retention`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "old.pdf", content = "a")

            val dashboard = assertNotNull(actual = library.dashboard(scan = null))

            assertTrue(actual = dashboard.tasks.none { it is OpenTask.RetentionDue })
        }
    }

    @Test
    fun `an ended retention is a task until it is marked as done, and then never again`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val first = library.add(path = "first.pdf", content = "a")
            val second = library.add(path = "second.pdf", content = "b")
            listOf(first, second).forEach {
                library.editDocumentDetails(
                    edit = DetailEdit.SetRetention(
                        documentId = it.id,
                        retention = RetentionModel(keepUntil = library.clock.today()),
                    ),
                )
            }

            val before = assertNotNull(
                actual = library.dashboard(scan = null),
            ).tasks.filterIsInstance<OpenTask.RetentionDue>()
            val settled = library.settleRetention(documentIds = listOf(first.id, second.id))
            val after = assertNotNull(
                actual = library.dashboard(scan = null),
            ).tasks.filterIsInstance<OpenTask.RetentionDue>()

            assertEquals(expected = 2, actual = before.size)
            assertIs<OperationOutcome.Done>(value = settled)
            assertTrue(actual = after.isEmpty())
            assertEquals(
                expected = library.clock.now(),
                actual = library.catalog.documents.getValue(key = first.id).retention?.settledAt,
            )
            assertNotNull(actual = library.catalog.documents.getValue(key = first.id).retention?.keepUntil)
        }
    }

    @Test
    fun `deadlines within their lead time are tasks and the next dates are upcoming`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.changeSettings(subject = "fields") { CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL) }
            val bill = library.add(path = "bill.pdf", content = "a")
            val due = library.clock.today().plusDays(5)
            library.editDocumentDetails(
                edit = DetailEdit.SetFields(
                    documentId = bill.id,
                    fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = due)),
                ),
            )

            val dashboard = assertNotNull(actual = library.dashboard(scan = null))

            assertEquals(
                expected = listOf(
                    OpenTask.DeadlineDue(documentId = bill.id, fieldKey = FieldPresets.DUE_DATE.key, date = due),
                ),
                actual = dashboard.tasks.filterIsInstance<OpenTask.DeadlineDue>(),
            )
            assertEquals(expected = listOf(due), actual = dashboard.upcoming.map { it.date })
        }
    }

    @Test
    fun `the calendar export has one all-day event per date with escaped text`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bill; final, v2.pdf", content = "a")
            library.editDocumentDetails(
                edit = DetailEdit.SetRetention(
                    documentId = bill.id,
                    retention = RetentionModel(keepUntil = library.clock.today().plusDays(3)),
                ),
            )

            val calendar = DeadlineCalendar.of(catalog = library.catalog, today = library.clock.today())

            assertTrue(actual = calendar.startsWith(prefix = "BEGIN:VCALENDAR\r\n"))
            assertTrue(actual = calendar.contains(other = "DTSTART;VALUE=DATE:20240304\r\n"))
            assertTrue(actual = calendar.contains(other = "SUMMARY:Retention ends: bill\\; final\\, v2.pdf\r\n"))
            assertEquals(expected = 1, actual = Regex(pattern = "BEGIN:VEVENT").findAll(input = calendar).count())
        }
    }
}
