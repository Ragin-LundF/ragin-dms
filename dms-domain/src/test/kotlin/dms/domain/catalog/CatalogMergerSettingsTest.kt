package dms.domain.catalog

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.field.FieldPresets
import dms.domain.link.model.DocumentLinkModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.rule.model.CorrespondentModel
import dms.domain.settings.model.LibraryLayoutModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class CatalogMergerSettingsTest {

    private val invoice = TestData.document(id = "invoice", path = "invoice.pdf")
    private val reminder = TestData.document(id = "reminder", path = "reminder.pdf")
    private val receipt = TestData.document(id = "receipt", path = "receipt.pdf")
    private val base = TestData.catalogWith(invoice, reminder, receipt)

    @Test
    fun `links of both copies are united by pair`() {
        val left = base.link(link = DocumentLinkModel.of(first = invoice.id, second = reminder.id)).applied()
        val right = base.link(link = DocumentLinkModel.of(first = receipt.id, second = invoice.id)).applied()

        val merged = CatalogMerger.merge(first = left, second = right).catalog

        assertEquals(expected = 2, actual = merged.links.size)
    }

    @Test
    fun `the newer copy's label wins for the same pair`() {
        val older = base.link(
            link = DocumentLinkModel.of(first = invoice.id, second = reminder.id, label = "old"),
        ).applied()
        val newer = older.link(
            link = DocumentLinkModel.of(first = invoice.id, second = reminder.id, label = "new"),
        ).applied()

        val merged = CatalogMerger.merge(first = newer, second = older).catalog

        assertEquals(expected = "new", actual = merged.links.single().label)
    }

    @Test
    fun `field definitions are merged per key and a clash is reported`() {
        val older = base.changeSettings(settings = base.settings.withField(definition = FieldPresets.AMOUNT)).applied()
        val renamed = FieldPresets.AMOUNT.copy(label = "Total")
        val newer = older.changeSettings(
            settings = older.settings.withField(definition = renamed).withField(definition = FieldPresets.DUE_DATE),
        ).applied()

        val result = CatalogMerger.merge(first = older, second = newer)

        assertEquals(
            expected = listOf(renamed, FieldPresets.DUE_DATE),
            actual = result.catalog.settings.fieldDefinitions,
        )
        val conflict = result.conflicts.single() as CatalogMergeConflict.FieldDefinitionDiffers
        assertEquals(expected = FieldPresets.AMOUNT, actual = conflict.discarded)
    }

    @Test
    fun `saved searches only in the older copy are kept`() {
        val search = SavedSearchModel(
            id = SavedSearchIdModel(value = "s1"),
            name = "Untagged",
            query = DocumentQueryModel(untagged = true),
        )
        val older = base.changeSettings(settings = base.settings.withSearch(search = search)).applied()
        val newer = base.setNote(documentId = invoice.id, note = "n").applied()
            .setNote(documentId = invoice.id, note = "m").applied()

        val result = CatalogMerger.merge(first = older, second = newer)

        assertEquals(expected = listOf(search), actual = result.catalog.settings.savedSearches)
        assertTrue(actual = result.conflicts.none { it is CatalogMergeConflict.SavedSearchDiffers })
    }

    @Test
    fun `correspondents are united by name with all aliases`() {
        val left = base.changeSettings(
            settings = base.settings.withCorrespondent(
                correspondent = CorrespondentModel(name = "Acme", aliases = listOf("ACME Ltd")),
            ),
        ).applied()
        val right = base.changeSettings(
            settings = base.settings.withCorrespondent(
                correspondent = CorrespondentModel(name = "acme", aliases = listOf("Acme Inc")),
            ),
        ).applied()

        val merged = CatalogMerger.merge(first = left, second = right).catalog

        assertEquals(
            expected = setOf("ACME Ltd", "Acme Inc"),
            actual = merged.settings.correspondents.single().aliases.toSet(),
        )
    }

    @Test
    fun `the newer layout wins`() {
        val older = base.changeSettings(
            settings = base.settings.withLayout(layout = LibraryLayoutModel(pathTemplate = "{year}")),
        )
            .applied()
        val newer = older.changeSettings(
            settings = older.settings.withLayout(layout = LibraryLayoutModel(pathTemplate = "{category}")),
        )
            .applied()

        val merged = CatalogMerger.merge(first = older, second = newer).catalog

        assertEquals(expected = "{category}", actual = merged.settings.layout.pathTemplate)
    }
}
