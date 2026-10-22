package dms.domain.catalog

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.link.model.DocumentLinkModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class CatalogLinksTest {

    private val invoice = TestData.document(id = "invoice", path = "invoice.pdf")
    private val reminder = TestData.document(id = "reminder", path = "reminder.pdf")
    private val receipt = TestData.document(id = "receipt", path = "receipt.pdf")
    private val catalog = TestData.catalogWith(invoice, reminder, receipt)

    @Test
    fun `a link is the same from both ends`() {
        val linked = catalog.link(link = DocumentLinkModel.of(first = reminder.id, second = invoice.id)).applied()

        assertEquals(expected = 1, actual = linked.linksOf(documentId = invoice.id).size)
        assertEquals(
            expected = invoice.id,
            actual = linked.linksOf(documentId = reminder.id).single().otherThan(reminder.id),
        )
        assertEquals(
            expected = DocumentLinkModel.of(first = invoice.id, second = reminder.id),
            actual = DocumentLinkModel.of(first = reminder.id, second = invoice.id),
        )
    }

    @Test
    fun `linking the same pair again replaces the label`() {
        val linked = catalog.link(
            link = DocumentLinkModel.of(first = invoice.id, second = reminder.id, label = "first"),
        )
            .applied()
            .link(link = DocumentLinkModel.of(first = reminder.id, second = invoice.id, label = "reminder")).applied()

        assertEquals(expected = "reminder", actual = linked.links.single().label)
    }

    @Test
    fun `a document cannot be linked to itself`() {
        assertFailsWith<IllegalArgumentException> { DocumentLinkModel.of(first = invoice.id, second = invoice.id) }
    }

    @Test
    fun `links to unknown documents are refused`() {
        val ghost = TestData.document(id = "ghost", path = "ghost.pdf")

        val update = catalog.link(link = DocumentLinkModel.of(first = invoice.id, second = ghost.id))

        assertEquals(expected = CatalogUpdate.Rejected(reason = CatalogRejection.DOCUMENT_NOT_FOUND), actual = update)
    }

    @Test
    fun `unlink removes the link from either end`() {
        val linked = catalog.link(link = DocumentLinkModel.of(first = invoice.id, second = reminder.id)).applied()

        val unlinked = linked.unlink(first = reminder.id, second = invoice.id).applied()

        assertTrue(actual = unlinked.links.isEmpty())
        assertEquals(
            expected = CatalogUpdate.Rejected(reason = CatalogRejection.LINK_NOT_FOUND),
            actual = unlinked.unlink(first = invoice.id, second = reminder.id),
        )
    }

    @Test
    fun `removing a document removes its links`() {
        val linked = catalog.link(link = DocumentLinkModel.of(first = invoice.id, second = reminder.id)).applied()
            .link(link = DocumentLinkModel.of(first = reminder.id, second = receipt.id)).applied()

        val removed = linked.removeDocument(documentId = reminder.id).applied()

        assertTrue(actual = removed.links.isEmpty())
    }
}
