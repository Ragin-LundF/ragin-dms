package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.document.model.DetailEdit
import dms.domain.catalog.CatalogRejection
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.retention.model.RetentionModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import kotlinx.coroutines.test.runTest
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class EditDocumentDetailsTest {

    @Test
    fun `notes, fields, retention and links are saved to every folder`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.changeSettings(subject = "fields") { CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL) }
            val bill = library.add(path = "bill.pdf", content = "a")
            val reminder = library.add(path = "reminder.pdf", content = "b")
            val due = FieldValue.Date(date = LocalDate.of(2024, 4, 1))

            listOf(
                DetailEdit.SetNote(documentId = bill.id, note = "Paid twice?\nAsk the supplier"),
                DetailEdit.SetFields(documentId = bill.id, fields = mapOf(FieldPresets.DUE_DATE.key to due)),
                DetailEdit.SetRetention(
                    documentId = bill.id,
                    retention = RetentionModel(keepUntil = LocalDate.of(2030, 1, 1)),
                ),
                DetailEdit.Link(first = reminder.id, second = bill.id, label = "reminder"),
            ).forEach { assertIs<OperationOutcome.Done>(value = library.editDocumentDetails(edit = it)) }

            library.allFolders.forEach { folder ->
                val saved = library.folders.catalogs.getValue(key = folder.id)
                val document = saved.documents.getValue(key = bill.id)
                assertEquals(expected = "Paid twice?\nAsk the supplier", actual = document.note)
                assertEquals(expected = due, actual = document.fields[FieldPresets.DUE_DATE.key])
                assertEquals(expected = LocalDate.of(2030, 1, 1), actual = document.retention?.keepUntil)
                assertEquals(expected = "reminder", actual = saved.linksOf(documentId = bill.id).single().label)
            }
        }
    }

    @Test
    fun `a field the library does not define is refused`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bill.pdf", content = "a")

            val outcome = library.editDocumentDetails(
                edit = DetailEdit.SetFields(
                    documentId = bill.id,
                    fields = mapOf(FieldPresets.AMOUNT.key to FieldValue.Flag(flag = true)),
                ),
            )

            assertEquals(
                expected = OperationOutcome.Rejected(reason = CatalogRejection.FIELD_NOT_DEFINED),
                actual = outcome,
            )
        }
    }

    @Test
    fun `a trashed document loses its links`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bill.pdf", content = "a")
            val reminder = library.add(path = "reminder.pdf", content = "b")
            library.editDocumentDetails(edit = DetailEdit.Link(first = bill.id, second = reminder.id, label = null))

            library.trashDocument(documentId = reminder.id, confirmed = true)

            assertTrue(actual = library.catalog.links.isEmpty())
        }
    }

    @Test
    fun `a category retention period is opt-in per category`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val tax = CategoryModel(id = CategoryIdModel(value = "tax"), name = "Tax")
            library.store.commit { it.addCategory(category = tax) }

            library.editDocumentDetails(edit = DetailEdit.SetRetentionYears(categoryId = tax.id, years = 10))

            assertEquals(expected = 10, actual = library.catalog.categories.getValue(key = tax.id).retentionYears)
        }
    }
}
