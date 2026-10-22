package dms.domain.catalog

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.retention.model.RetentionModel
import dms.domain.settings.model.CatalogSettingsModel
import java.math.BigDecimal
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertNull

class CatalogDetailsTest {

    private val invoice = TestData.document(id = "invoice", path = "letters/invoice.pdf")
    private val reminder = TestData.document(id = "reminder", path = "letters/reminder.pdf")
    private val amount = FieldValue.Money(amount = BigDecimal("120.50"), currency = CurrencyCodeModel(value = "EUR"))
    private val withPresets = TestData.catalogWith(invoice, reminder)
        .changeSettings(settings = CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL)).applied()

    @Test
    fun `field values need a definition`() {
        val update = TestData.catalogWith(invoice).setFields(
            documentId = invoice.id,
            fields = mapOf(FieldKeyModel(value = "amount") to amount),
        )

        assertEquals(expected = CatalogUpdate.Rejected(reason = CatalogRejection.FIELD_NOT_DEFINED), actual = update)
    }

    @Test
    fun `field values must have the type of their definition`() {
        val update = withPresets.setFields(
            documentId = invoice.id,
            fields = mapOf(FieldPresets.AMOUNT.key to FieldValue.Date(date = LocalDate.of(2026, 1, 1))),
        )

        assertEquals(expected = CatalogUpdate.Rejected(reason = CatalogRejection.FIELD_TYPE_MISMATCH), actual = update)
    }

    @Test
    fun `field values of the right type are stored with the next revision`() {
        val catalog = withPresets.setFields(documentId = invoice.id, fields = mapOf(FieldPresets.AMOUNT.key to amount))
            .applied()

        assertEquals(
            expected = mapOf(FieldPresets.AMOUNT.key to amount),
            actual = catalog.documents.getValue(invoice.id).fields,
        )
        assertEquals(expected = withPresets.revision.next(), actual = catalog.revision)
    }

    @Test
    fun `a blank note removes the note`() {
        val noted = withPresets.setNote(documentId = invoice.id, note = "Paid by transfer\nsecond line").applied()
        val cleared = noted.setNote(documentId = invoice.id, note = "   ").applied()

        assertEquals(expected = "Paid by transfer\nsecond line", actual = noted.documents.getValue(invoice.id).note)
        assertNull(actual = cleared.documents.getValue(invoice.id).note)
    }

    @Test
    fun `an empty retention is stored as none`() {
        val kept = withPresets.setRetention(
            documentId = invoice.id,
            retention = RetentionModel(keepUntil = LocalDate.of(2030, 12, 31)),
        ).applied()
        val cleared = kept.setRetention(documentId = invoice.id, retention = RetentionModel()).applied()

        assertEquals(
            expected = LocalDate.of(2030, 12, 31),
            actual = kept.documents.getValue(invoice.id).retention?.keepUntil,
        )
        assertNull(actual = cleared.documents.getValue(invoice.id).retention)
    }

    @Test
    fun `many documents move in one revision and may swap paths`() {
        val moved = withPresets.moveDocuments(
            moves = mapOf(invoice.id to reminder.path, reminder.id to invoice.path),
        ).applied()

        assertEquals(expected = reminder.path, actual = moved.documents.getValue(invoice.id).path)
        assertEquals(expected = invoice.path, actual = moved.documents.getValue(reminder.id).path)
        assertEquals(expected = withPresets.revision.next(), actual = moved.revision)
    }

    @Test
    fun `a batch move that puts two documents on one path is refused`() {
        val update = withPresets.moveDocuments(moves = mapOf(invoice.id to reminder.path))

        assertEquals(expected = CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED), actual = update)
    }

    @Test
    fun `a batch move of an unknown document is refused`() {
        val update = withPresets.moveDocuments(
            moves = mapOf(TestData.document(id = "ghost", path = "x.pdf").id to RelativePathModel.of(raw = "y.pdf")),
        )

        assertIs<CatalogUpdate.Rejected>(value = update)
    }

    @Test
    fun `retention years of a category are opt-in and checked`() {
        val category = dms.domain.taxonomy.model.CategoryModel(
            id = dms.domain.taxonomy.model.CategoryIdModel(value = "tax"),
            name = "Tax",
        )
        val catalog = withPresets.addCategory(category = category).applied()
            .setRetentionYears(categoryId = category.id, years = 10).applied()

        assertEquals(expected = 10, actual = catalog.categories.getValue(category.id).retentionYears)
        assertNull(actual = category.retentionYears)
        assertFailsWith<IllegalArgumentException> { category.copy(retentionYears = 0) }
    }
}
