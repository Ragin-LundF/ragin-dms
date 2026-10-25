package dms.application.export

import dms.application.TestLibrary
import dms.application.export.model.ExportOptionsModel
import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.IndexRowModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldType
import dms.domain.field.FieldValue
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.math.BigDecimal
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class ExportPlannerTest {

    private val finance = CategoryModel(id = CategoryIdModel(value = "finance"), name = "Finance")
    private val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills", parentId = finance.id)
    private val amount = FieldDefinitionModel(
        key = FieldKeyModel(value = "amount"),
        label = "Amount",
        type = FieldType.NUMBER,
    )
    private val due = FieldDefinitionModel(key = FieldKeyModel(value = "due"), label = "Due", type = FieldType.DATE)
    private val catalog = CatalogModel(
        libraryId = LibraryIdModel(value = "library"),
        revision = CatalogRevisionModel.INITIAL,
        documents = emptyMap(),
        categories = mapOf(finance.id to finance, bills.id to bills),
        tags = emptySet(),
        settings = CatalogSettingsModel(fieldDefinitions = listOf(amount, due)),
    )

    @Test
    fun `keeps the folders and sorts the files by their path`() {
        val documents = listOf(
            document(id = "1", path = "2025/scan.pdf"),
            document(id = "2", path = "2024/scan.pdf"),
            document(id = "3", path = "top.pdf"),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel())

        assertEquals(
            expected = listOf(
                "2024/scan.pdf" to "2024/scan.pdf",
                "2025/scan.pdf" to "2025/scan.pdf",
                "top.pdf" to "top.pdf",
            ),
            actual = bundle.entries.map { it.source.value to it.target },
        )
    }

    @Test
    fun `flattening puts every file at the top and suffixes clashes in path order, ignoring case`() {
        val documents = listOf(
            document(id = "1", path = "b/Scan.pdf"),
            document(id = "2", path = "a/scan.pdf"),
            document(id = "3", path = "c/SCAN.pdf"),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel(flatten = true))

        assertEquals(
            expected = listOf("scan.pdf", "Scan (2).pdf", "SCAN (3).pdf"),
            actual = bundle.entries.map { it.target },
        )
    }

    @Test
    fun `folders that differ only by case share their names`() {
        val documents = listOf(
            document(id = "1", path = "Letters/scan.pdf"),
            document(id = "2", path = "letters/Scan.pdf"),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel())

        assertEquals(
            expected = listOf("Letters/scan.pdf", "letters/Scan (2).pdf"),
            actual = bundle.entries.map { it.target },
        )
    }

    @Test
    fun `renaming to the title makes it a valid name and keeps the extension`() {
        val documents = listOf(
            document(id = "1", path = "letters/scan.pdf", title = "Power: bill?"),
            document(id = "2", path = "letters/untitled.pdf"),
            document(id = "3", path = "letters/notes", title = "Plain notes"),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel(renameToTitle = true))

        assertEquals(
            expected = listOf("letters/Plain notes", "letters/Power- bill-.pdf", "letters/untitled.pdf"),
            actual = bundle.entries.map { it.target },
        )
    }

    @Test
    fun `a long title is cut so that the name with its extension stays within the limit`() {
        val documents = listOf(document(id = "1", path = "scan.pdf", title = "x".repeat(n = 300)))

        val bundle = plan(documents = documents, options = ExportOptionsModel(renameToTitle = true))

        assertEquals(expected = "x".repeat(n = 196) + ".pdf", actual = bundle.entries.single().target)
    }

    @Test
    fun `documents with the same title get a suffix`() {
        val documents = listOf(
            document(id = "1", path = "letters/a.pdf", title = "Bill"),
            document(id = "2", path = "letters/b.pdf", title = "bill"),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel(renameToTitle = true))

        assertEquals(
            expected = listOf("letters/Bill.pdf", "letters/bill (2).pdf"),
            actual = bundle.entries.map { it.target },
        )
    }

    @Test
    fun `the index describes every file under its exported path`() {
        val documents = listOf(
            document(id = "1", path = "letters/scan.pdf", title = "Power bill").copy(
                metadata = DocumentMetadataModel(
                    title = "Power bill",
                    documentDate = LocalDate.of(2024, 2, 1),
                    correspondent = "Sample Energy",
                ),
                categoryId = bills.id,
                tags = setOf(TagNameModel.of(raw = "tax"), TagNameModel.of(raw = "Energy")),
                fields = mapOf(due.key to FieldValue.Date(date = LocalDate.of(2024, 3, 1))),
                note = "paid",
            ),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel(flatten = true))

        assertEquals(
            expected = listOf(
                IndexRowModel(
                    path = "scan.pdf",
                    title = "Power bill",
                    date = "2024-02-01",
                    correspondent = "Sample Energy",
                    category = "Finance / Bills",
                    tags = "Energy, tax",
                    fields = listOf("", "2024-03-01"),
                    note = "paid",
                ),
            ),
            actual = bundle.index,
        )
        assertEquals(expected = listOf("Amount", "Due"), actual = bundle.fieldLabels)
    }

    @Test
    fun `a document without details has empty columns and the day it was added in the given zone`() {
        val documents = listOf(
            document(id = "1", path = "scan.pdf").copy(
                addedAt = Instant.parse("2024-01-01T23:30:00Z"),
                fields = mapOf(amount.key to FieldValue.Number(number = BigDecimal("12.50"))),
            ),
        )

        val bundle = ExportPlanner.plan(
            catalog = catalog,
            documents = documents,
            options = ExportOptionsModel(),
            zone = ZoneId.of("Europe/Berlin"),
        )

        assertEquals(
            expected = listOf(
                IndexRowModel(
                    path = "scan.pdf",
                    title = "",
                    date = "2024-01-02",
                    correspondent = "",
                    category = "",
                    tags = "",
                    fields = listOf("12.50", ""),
                    note = "",
                ),
            ),
            actual = bundle.index,
        )
    }

    @Test
    fun `a category the catalog does not know leaves the column empty`() {
        val documents = listOf(
            document(id = "1", path = "scan.pdf").copy(categoryId = CategoryIdModel(value = "gone")),
        )

        val bundle = plan(documents = documents, options = ExportOptionsModel())

        assertEquals(expected = listOf(""), actual = bundle.index?.map { it.category })
    }

    @Test
    fun `without an index the bundle holds only the files`() {
        val bundle = plan(
            documents = listOf(document(id = "1", path = "scan.pdf")),
            options = ExportOptionsModel(index = false),
        )

        assertNull(actual = bundle.index)
        assertEquals(expected = listOf("scan.pdf"), actual = bundle.entries.map { it.target })
    }

    @Test
    fun `no documents make an empty bundle`() {
        val bundle = plan(documents = emptyList(), options = ExportOptionsModel())

        assertEquals(expected = emptyList(), actual = bundle.entries)
        assertEquals(expected = emptyList(), actual = bundle.index)
    }

    private fun plan(documents: List<DocumentModel>, options: ExportOptionsModel): ExportBundleModel {
        return ExportPlanner.plan(catalog = catalog, documents = documents, options = options, zone = ZoneId.of("UTC"))
    }

    private fun document(id: String, path: String, title: String? = null): DocumentModel {
        val document = TestLibrary.document(id = id, path = path, content = id)
        return document.copy(metadata = DocumentMetadataModel(title = title))
    }
}
