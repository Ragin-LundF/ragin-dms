package dms.domain.deadline

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.retention.model.RetentionModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.task.OpenTask
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class DeadlinePlannerTest {

    private val today = LocalDate.of(2026, 3, 1)
    private val contract = TestData.document(id = "contract", path = "contract.pdf")
    private val bill = TestData.document(id = "bill", path = "bill.pdf")
    private val base = TestData.catalogWith(contract, bill)
        .changeSettings(settings = CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL)).applied()

    private fun withDates(cancelBy: LocalDate, due: LocalDate): dms.domain.catalog.model.CatalogModel {
        return base
            .setFields(
                documentId = contract.id,
                fields = mapOf(FieldPresets.CANCEL_BY.key to FieldValue.Date(date = cancelBy)),
            )
            .applied()
            .setFields(documentId = bill.id, fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = due)))
            .applied()
    }

    @Test
    fun `deadlines appear within the lead time of their role`() {
        val catalog = withDates(cancelBy = today.plusDays(30), due = today.plusDays(8))

        val tasks = DeadlinePlanner.dueTasks(catalog = catalog, today = today)

        assertEquals(
            expected = listOf(
                OpenTask.DeadlineDue(
                    documentId = contract.id,
                    fieldKey = FieldPresets.CANCEL_BY.key,
                    date = today.plusDays(30),
                ),
            ),
            actual = tasks,
        )
    }

    @Test
    fun `the task key changes with the date, so a dismissed task returns`() {
        val first = DeadlinePlanner.dueTasks(
            catalog = withDates(cancelBy = today.plusDays(3), due = today.plusDays(1)),
            today = today,
        )
        val moved = DeadlinePlanner.dueTasks(
            catalog = withDates(cancelBy = today.plusDays(4), due = today.plusDays(1)),
            today = today,
        )

        assertNotEquals(illegal = first.first().key, actual = moved.first().key)
        assertEquals(expected = first.last().key, actual = moved.last().key)
    }

    @Test
    fun `a retention task appears once its date is reached`() {
        val catalog = base
            .setRetention(documentId = contract.id, retention = RetentionModel(keepUntil = today)).applied()
            .setRetention(documentId = bill.id, retention = RetentionModel(keepUntil = today.plusDays(1))).applied()

        val tasks = DeadlinePlanner.dueTasks(catalog = catalog, today = today)

        assertEquals(expected = listOf(OpenTask.RetentionDue(documentId = contract.id, date = today)), actual = tasks)
    }

    @Test
    fun `upcoming lists the next dates in order`() {
        val catalog = withDates(cancelBy = today.plusDays(90), due = today.plusDays(2))
            .setRetention(documentId = bill.id, retention = RetentionModel(keepUntil = today.minusDays(1))).applied()

        val upcoming = DeadlinePlanner.upcoming(catalog = catalog, today = today, limit = 10)

        assertEquals(expected = listOf(today.plusDays(2), today.plusDays(90)), actual = upcoming.map { it.date })
        assertTrue(actual = DeadlinePlanner.upcoming(catalog = catalog, today = today, limit = 1).size == 1)
    }
}
