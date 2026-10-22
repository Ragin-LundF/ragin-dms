package dms.application.export

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.application.document.model.DetailEdit
import dms.application.export.model.ExportOptionsModel
import dms.application.export.model.ExportOutcome
import dms.application.export.model.ExportScope
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class ExportDocumentsTest {

    @Test
    fun `flattening gives clashing names a suffix, ignoring case`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val first = library.add(path = "2024/scan.pdf", content = "a")
            val second = library.add(path = "2025/SCAN.pdf", content = "b")

            val outcome = library.exportDocuments(
                scope = ExportScope.Documents(ids = listOf(first.id, second.id)),
                directory = "/outside/export",
                name = "Bundle",
                options = ExportOptionsModel(flatten = true),
            )

            assertEquals(
                expected = ExportOutcome.Exported(count = 2, location = "/outside/export/Bundle"),
                actual = outcome,
            )
            val (bundle, target) = library.exports.written.single()
            assertEquals(expected = listOf("scan.pdf", "SCAN (2).pdf"), actual = bundle.entries.map { it.target })
            assertEquals(expected = false, actual = target.zip)
        }
    }

    @Test
    fun `renaming to the title keeps the extension and the index carries every value`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.changeSettings(subject = "fields") {
                CatalogSettingsModel(fieldDefinitions = listOf(FieldPresets.DUE_DATE))
            }
            val document = library.add(path = "letters/scan.pdf", content = "a")
            library.editCatalog(
                edit = CatalogEdit.UpdateMetadata(
                    documentId = document.id,
                    metadata = DocumentMetadataModel(
                        title = "Power: bill",
                        documentDate = LocalDate.of(2024, 2, 1),
                        correspondent = "Sample Energy",
                    ),
                ),
            )
            library.editCatalog(
                edit = CatalogEdit.AddTags(documentId = document.id, tags = setOf(TagNameModel.of(raw = "Tax"))),
            )
            library.editDocumentDetails(
                edit = DetailEdit.SetFields(
                    documentId = document.id,
                    fields = mapOf(FieldPresets.DUE_DATE.key to FieldValue.Date(date = LocalDate.of(2024, 3, 1))),
                ),
            )
            library.editDocumentDetails(edit = DetailEdit.SetNote(documentId = document.id, note = "paid"))

            library.exportDocuments(
                scope = ExportScope.Query(
                    query = DocumentQueryModel(
                        tags = dms.domain.query.model.TagFilterModel(any = setOf(TagNameModel.of(raw = "tax"))),
                    ),
                ),
                directory = "/outside",
                name = "Tax 2024",
                options = ExportOptionsModel(renameToTitle = true, zip = true),
            )

            val (bundle, target) = library.exports.written.single()
            assertEquals(expected = listOf("letters/Power- bill.pdf"), actual = bundle.entries.map { it.target })
            assertEquals(
                expected = RelativePathModel.of(raw = "letters/scan.pdf"),
                actual = bundle.entries.single().source,
            )
            val row = bundle.index.orEmpty().single()
            assertEquals(
                expected = listOf("2024-02-01", "Sample Energy", "Tax", "2024-03-01", "paid"),
                actual = listOf(row.date, row.correspondent, row.tags, row.fields.single(), row.note),
            )
            assertEquals(expected = listOf("Due date"), actual = bundle.fieldLabels)
            assertEquals(expected = true, actual = target.zip)
        }
    }

    @Test
    fun `an export never writes into a library folder and an empty scope writes nothing`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "a.pdf", content = "a")

            val inside = library.exportDocuments(
                scope = ExportScope.Documents(ids = listOf(document.id)),
                directory = library.syncA.location.value + "/exports",
                name = "x",
                options = ExportOptionsModel(),
            )
            val empty = library.exportDocuments(
                scope = ExportScope.Query(query = DocumentQueryModel(text = "nothing matches this")),
                directory = "/outside",
                name = "x",
                options = ExportOptionsModel(),
            )

            assertEquals(expected = ExportOutcome.InsideLibrary, actual = inside)
            assertIs<ExportOutcome.Empty>(value = empty)
            assertEquals(expected = 0, actual = library.exports.written.size)
        }
    }
}
