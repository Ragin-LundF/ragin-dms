package dms.domain.catalog

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class CatalogDocumentTest {

    private val invoice = TestData.document(id = "d1", path = "projects/alpha/invoice.pdf")
    private val contract = TestData.document(id = "d2", path = "projects/alpha/contract.pdf")
    private val notes = TestData.document(id = "d3", path = "projects/beta/notes.docx")

    @Test
    fun `every applied change increments the revision`() {
        val catalog = TestData.catalogWith(invoice, contract)

        assertEquals(expected = CatalogRevisionModel(value = 2), actual = catalog.revision)
    }

    @Test
    fun `a document id or path can only be used once`() {
        val catalog = TestData.catalogWith(invoice)

        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_ALREADY_EXISTS),
            actual = catalog.addDocument(document = invoice.copy(path = RelativePathModel.of(raw = "other.pdf"))),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED),
            actual = catalog.addDocument(document = contract.copy(path = invoice.path)),
        )
    }

    @Test
    fun `a document with an unknown category is rejected`() {
        val document = invoice.copy(categoryId = CategoryIdModel(value = "missing"))

        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.CATEGORY_NOT_FOUND),
            actual = CatalogModel.empty(libraryId = TestData.LIBRARY_ID).addDocument(document = document),
        )
    }

    @Test
    fun `moving a document onto another document's path is rejected`() {
        val catalog = TestData.catalogWith(invoice, contract)

        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED),
            actual = catalog.moveDocument(documentId = invoice.id, newPath = contract.path),
        )
    }

    @Test
    fun `moving a document changes its path and keeps its id`() {
        val newPath = RelativePathModel.of(raw = "archive/invoice.pdf")

        val moved = TestData.catalogWith(invoice).moveDocument(documentId = invoice.id, newPath = newPath).applied()

        assertEquals(expected = invoice.id, actual = moved.findByPath(path = newPath)?.id)
        assertNull(actual = moved.findByPath(path = invoice.path))
    }

    @Test
    fun `moving a folder relocates every document below it`() {
        val catalog = TestData.catalogWith(invoice, contract, notes)

        val moved = catalog.moveFolder(
            from = RelativePathModel.of(raw = "projects/alpha"),
            to = RelativePathModel.of(raw = "archive/alpha"),
        ).applied()

        assertEquals(
            expected = setOf("archive/alpha/invoice.pdf", "archive/alpha/contract.pdf", "projects/beta/notes.docx"),
            actual = moved.documents.values.map { it.path.value }.toSet(),
        )
    }

    @Test
    fun `moving a folder into itself or onto occupied paths is rejected`() {
        val catalog = TestData.catalogWith(
            invoice,
            notes.copy(path = RelativePathModel.of(raw = "archive/invoice.pdf")),
        )
        val alpha = RelativePathModel.of(raw = "projects/alpha")

        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.INVALID_FOLDER_MOVE),
            actual = catalog.moveFolder(from = alpha, to = RelativePathModel.of(raw = "projects/alpha/sub")),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.INVALID_FOLDER_MOVE),
            actual = catalog.moveFolder(from = RelativePathModel.ROOT, to = alpha),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.PATH_ALREADY_USED),
            actual = catalog.moveFolder(from = alpha, to = RelativePathModel.of(raw = "archive")),
        )
    }

    @Test
    fun `changes to an unknown document are rejected`() {
        val catalog = TestData.catalogWith(invoice)
        val unknown = DocumentIdModel(value = "unknown")

        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND),
            actual = catalog.updateMetadata(documentId = unknown, metadata = DocumentMetadataModel.EMPTY),
        )
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND),
            actual = catalog.removeDocument(documentId = unknown),
        )
    }

    @Test
    fun `metadata and content updates replace the stored values`() {
        val metadata = DocumentMetadataModel(title = "Example invoice", documentDate = LocalDate.of(2024, 1, 15))
        val catalog = TestData.catalogWith(invoice)
            .updateMetadata(documentId = invoice.id, metadata = metadata).applied()
            .updateContent(documentId = invoice.id, contentHash = TestData.hash(seed = 42), size = 7).applied()

        val updated = catalog.documents.getValue(key = invoice.id)
        assertEquals(expected = metadata, actual = updated.metadata)
        assertEquals(expected = TestData.hash(seed = 42), actual = updated.contentHash)
        assertEquals(expected = 7, actual = updated.size)
    }

    @Test
    fun `setting tags registers new tags with the catalog's existing spelling`() {
        val catalog = TestData.catalogWith(invoice, contract)
            .setTags(documentId = invoice.id, newTags = setOf(TagNameModel.of(raw = "Tax"))).applied()
            .setTags(
                documentId = contract.id,
                newTags = setOf(TagNameModel.of(raw = "tax"), TagNameModel.of(raw = "Car")),
            )
            .applied()

        assertEquals(expected = listOf("Car", "Tax"), actual = catalog.tags.map { it.value }.sorted())
        assertEquals(
            expected = setOf("Tax", "Car"),
            actual = catalog.documents.getValue(key = contract.id).tags.map { it.value }.toSet(),
        )
    }

    @Test
    fun `chained updates stop at the first rejection`() {
        val catalog = TestData.catalogWith(invoice)

        val chained = catalog.removeDocument(documentId = DocumentIdModel(value = "unknown"))
            .andThen { it.removeDocument(documentId = invoice.id) }

        assertEquals(expected = CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND), actual = chained)
    }

    @Test
    fun `documents inside a folder include nested folders`() {
        val catalog = TestData.catalogWith(invoice, contract, notes)

        val inside = catalog.documentsInside(folder = RelativePathModel.of(raw = "projects/alpha"))

        assertEquals(expected = setOf(invoice.id, contract.id), actual = inside.map { it.id }.toSet())
    }

    @Test
    fun `documents can be found by content hash`() {
        val copy = contract.copy(id = DocumentIdModel(value = "copy"), path = RelativePathModel.of(raw = "copy.pdf"))
        val catalog = TestData.catalogWith(invoice, contract, copy)

        assertEquals(
            expected = setOf(contract.id, copy.id),
            actual = catalog.findByContentHash(contentHash = contract.contentHash).map { it.id }.toSet(),
        )
    }
}
