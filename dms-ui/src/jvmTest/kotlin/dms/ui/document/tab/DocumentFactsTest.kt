package dms.ui.document.tab

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
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class DocumentFactsTest {

    private val today = LocalDate.parse("2026-03-10")
    private val catalog = CatalogModel.empty(libraryId = LibraryIdModel(value = "sample-library"))
        .copy(settings = CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL))

    @Test
    fun `a due date within its lead time gets an urgent chip with the days left`() {
        val facts = factsOf(fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = today.plusDays(5))))

        assertEquals(
            expected = DocumentFactModel(label = "Due date", value = "2026-03-15", chip = "in 5 days", urgent = true),
            actual = facts.single { it.label == "Due date" },
        )
    }

    @Test
    fun `a deadline further away gets a calm chip, and a past one an urgent chip`() {
        val later = factsOf(fields = mapOf(FieldPresets.CANCEL_BY.key to FieldValue.Date(date = today.plusDays(60))))
        val past = factsOf(fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = today.minusDays(3))))

        assertEquals(
            expected = DocumentFactModel(
                label = "Cancel by",
                value = "2026-05-09",
                chip = "in 2 months",
                urgent = false,
            ),
            actual = later.single { it.label == "Cancel by" },
        )
        assertEquals(
            expected = DocumentFactModel(label = "Due date", value = "2026-03-07", chip = "3 days ago", urgent = true),
            actual = past.single { it.label == "Due date" },
        )
    }

    @Test
    fun `fields without a value are left out, and missing details say so`() {
        val facts = factsOf(fields = emptyMap())

        assertEquals(
            expected = listOf("Category", "Tags", "Folder", "Size", "Added", "Retention"),
            actual = facts.map { it.label },
        )
        assertEquals(
            expected = listOf("None", "None", "letters", "12 B · PDF"),
            actual = facts.take(n = 4).map { it.value },
        )
        assertEquals(expected = "Not set", actual = facts.last().value)
    }

    private fun factsOf(fields: Map<FieldKeyModel, FieldValue>): List<DocumentFactModel> {
        val document = DocumentModel(
            id = DocumentIdModel(value = "doc-1"),
            path = RelativePathModel.of(raw = "letters/sample.pdf"),
            contentHash = ContentHashModel(value = "a".repeat(n = 64)),
            size = 12,
            addedAt = Instant.parse("2026-01-02T10:00:00Z"),
            fields = fields,
        )
        return DocumentFacts.of(document = document, catalog = catalog, today = today, zone = ZoneOffset.UTC)
    }
}
