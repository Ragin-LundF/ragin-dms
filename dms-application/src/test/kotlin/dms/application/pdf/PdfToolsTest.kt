package dms.application.pdf

import dms.application.TestLibrary
import dms.application.catalog.model.CatalogEdit
import dms.application.common.OperationOutcome
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.storage.model.FolderOperation
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class PdfToolsTest {

    @Test
    fun `editing pages makes a new version everywhere, and the old one goes to the trash`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "letters/scan.pdf", content = "one|two|three")

            val outcome = library.editPages(
                documentId = scan.id,
                pages = listOf(PageChangeModel(page = 2, rotation = 90), PageChangeModel(page = 0)),
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "three@90|one",
                    actual = library.folders.content(folder = folder, path = "letters/scan.pdf"),
                )
                assertTrue(actual = folder.id to scan.path in library.folders.trashed)
            }
            val edited = library.catalog.documents.getValue(key = scan.id)
            assertEquals(
                expected = dms.application.fake.storage.FakeFolders.hashOf(content = "three@90|one"),
                actual = edited.contentHash,
            )
            assertEquals(expected = setOf("letters/scan.pdf"), actual = library.folders.paths(folder = library.primary))
        }
    }

    @Test
    fun `an offline sync folder gets the new version from the journal`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "one|two")
            library.folders.unreachable += library.syncB.id

            library.editPages(documentId = scan.id, pages = listOf(PageChangeModel(page = 1)))

            assertTrue(
                actual = library.journal.pending().any {
                    it.operation == FolderOperation.CopyFromPrimary(path = scan.path, replaceExisting = true)
                },
            )
        }
    }

    @Test
    fun `splitting creates documents that inherit category, tags and correspondent, and keeps the original`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bills = CategoryModel(id = CategoryIdModel(value = "bills"), name = "Bills")
            library.store.commit { it.addCategory(category = bills) }
            val scan = library.add(path = "letters/batch.pdf", content = "a|b|c")
            library.editCatalog(edit = CatalogEdit.AssignCategory(documentId = scan.id, categoryId = bills.id))
            library.editCatalog(
                edit = CatalogEdit.AddTags(documentId = scan.id, tags = setOf(TagNameModel.of(raw = "Tax"))),
            )
            library.editCatalog(
                edit = CatalogEdit.UpdateMetadata(
                    documentId = scan.id,
                    metadata = DocumentMetadataModel(correspondent = "Sample Energy"),
                ),
            )

            val outcome = library.splitDocument(
                documentId = scan.id,
                ranges = listOf(0..0, 1..2),
                trashOriginal = false,
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            val parts = library.catalog.documents.values.filter { it.id != scan.id }.sortedBy { it.path.value }
            assertEquals(
                expected = listOf("letters/batch (page 1).pdf", "letters/batch (pages 2-3).pdf"),
                actual = parts.map { it.path.value },
            )
            parts.forEach { part ->
                assertEquals(expected = bills.id, actual = part.categoryId)
                assertEquals(expected = setOf(TagNameModel.of(raw = "Tax")), actual = part.tags)
                assertEquals(expected = "Sample Energy", actual = part.metadata.correspondent)
            }
            library.allFolders.forEach { folder ->
                val content = library.folders.content(folder = folder, path = "letters/batch (pages 2-3).pdf")
                assertEquals(expected = "b|c", actual = content)
            }
            assertTrue(actual = scan.id in library.catalog.documents)
        }
    }

    @Test
    fun `splitting with the confirmed choice moves the original to the trash`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "batch.pdf", content = "a|b")

            library.splitDocument(documentId = scan.id, ranges = listOf(0..0, 1..1), trashOriginal = true)

            assertTrue(actual = scan.id !in library.catalog.documents)
            assertEquals(expected = 2, actual = library.catalog.documents.size)
        }
    }

    @Test
    fun `merging keeps the order and trashes the sources only when confirmed`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val first = library.add(path = "first.pdf", content = "1")
            val second = library.add(path = "second.pdf", content = "2")

            library.mergeDocuments(documentIds = listOf(second.id, first.id), fileName = "both", trashSources = false)

            assertEquals(expected = "2|1", actual = library.folders.content(folder = library.syncA, path = "both.pdf"))
            assertEquals(expected = 3, actual = library.catalog.documents.size)

            library.mergeDocuments(
                documentIds = listOf(first.id, second.id),
                fileName = "both.pdf",
                trashSources = true,
            )

            assertEquals(
                expected = "1|2",
                actual = library.folders.content(folder = library.primary, path = "both (2).pdf"),
            )
            assertEquals(
                expected = setOf("both.pdf", "both (2).pdf"),
                actual = library.catalog.documents.values.map { it.path.value }.toSet(),
            )
        }
    }
}
