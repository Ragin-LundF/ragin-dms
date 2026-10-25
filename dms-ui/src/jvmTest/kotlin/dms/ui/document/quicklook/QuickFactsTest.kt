package dms.ui.document.quicklook

import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.model.RelativePathModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.ui.document.model.DocumentFactModel
import dms.ui.document.model.QuickFactsModel
import java.time.Instant
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class QuickFactsTest {

    private val today = LocalDate.parse("2026-03-10")
    private val empty = CatalogModel.empty(libraryId = LibraryIdModel(value = "sample-library"))
    private val withFields = empty.copy(settings = CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL))

    @Test
    fun `a due date within its lead time gets an urgent chip with the days left`() {
        val facts = factsOf(fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = today.plusDays(5))))

        assertEquals(
            expected = DocumentFactModel(label = "Due date", value = "2026-03-15", chip = "in 5 days", urgent = true),
            actual = facts.due,
        )
    }

    @Test
    fun `a due date further away gets a calm chip, and a past one an urgent chip`() {
        val later = factsOf(fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = today.plusDays(60))))
        val past = factsOf(fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = today.minusDays(3))))

        assertEquals(
            expected = DocumentFactModel(label = "Due date", value = "2026-05-09", chip = "in 2 months"),
            actual = later.due,
        )
        assertEquals(
            expected = DocumentFactModel(label = "Due date", value = "2026-03-07", chip = "3 days ago", urgent = true),
            actual = past.due,
        )
    }

    @Test
    fun `amount and due show without a value while the library has the fields, and not at all without them`() {
        val withoutValues = factsOf(fields = emptyMap())
        val withoutFields = factsOf(fields = emptyMap(), catalog = empty)

        assertEquals(
            expected = QuickFactsModel(
                category = null,
                tags = emptyList(),
                correspondent = null,
                documentDate = null,
                amount = DocumentFactModel(label = "Amount", value = null),
                due = DocumentFactModel(label = "Due date", value = null),
            ),
            actual = withoutValues,
        )
        assertNull(actual = withoutFields.amount)
        assertNull(actual = withoutFields.due)
    }

    @Test
    fun `other deadlines are no key fact`() {
        val facts = factsOf(fields = mapOf(FieldPresets.CANCEL_BY.key to FieldValue.Date(date = today.plusDays(5))))

        assertEquals(expected = DocumentFactModel(label = "Due date", value = null), actual = facts.due)
    }

    private fun factsOf(fields: Map<FieldKeyModel, FieldValue>, catalog: CatalogModel = withFields): QuickFactsModel {
        val document = DocumentModel(
            id = DocumentIdModel(value = "doc-1"),
            path = RelativePathModel.of(raw = "letters/sample.pdf"),
            contentHash = ContentHashModel(value = "a".repeat(n = 64)),
            size = 12,
            addedAt = Instant.parse("2026-01-02T10:00:00Z"),
            fields = fields,
        )
        return QuickFacts.of(document = document, catalog = catalog, today = today)
    }
}
