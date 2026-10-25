package dms.ui.document.quicklook

import dms.domain.document.model.DocumentIdModel
import dms.ui.document.model.QuickFact
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class QuickLookControllerTest {

    private val first = DocumentIdModel(value = "doc-1")
    private val second = DocumentIdModel(value = "doc-2")
    private val third = DocumentIdModel(value = "doc-3")
    private val order = listOf(first, second, third)

    @Test
    fun `Space opens the quick look on the chosen document and closes it again`() {
        val quickLook = QuickLookController()

        quickLook.toggle(documentId = second, order = order)
        val opened = quickLook.state.value.shown
        quickLook.toggle(documentId = second, order = order)

        assertEquals(expected = second, actual = opened)
        assertFalse(actual = quickLook.isOpen)
        assertNull(actual = quickLook.state.value.shown)
    }

    @Test
    fun `Space without a chosen document opens nothing`() {
        val quickLook = QuickLookController()

        quickLook.toggle(documentId = null, order = order)

        assertFalse(actual = quickLook.isOpen)
    }

    @Test
    fun `down and up move through the handed-in order and stop at its ends`() {
        val quickLook = QuickLookController()
        quickLook.open(documentId = first, order = order)

        quickLook.previous()
        val atStart = quickLook.state.value.shown
        quickLook.next()
        val afterOne = quickLook.state.value.shown
        quickLook.next()
        quickLook.next()
        val atEnd = quickLook.state.value.shown
        quickLook.previous()

        assertEquals(expected = listOf(first, second, third), actual = listOf(atStart, afterOne, atEnd))
        assertEquals(expected = second, actual = quickLook.state.value.shown)
    }

    @Test
    fun `a new order counts for the next step, and a document that left the view stays until closed`() {
        val quickLook = QuickLookController()
        quickLook.open(documentId = second, order = order)

        quickLook.updateOrder(order = listOf(third, second, first))
        quickLook.next()
        val reordered = quickLook.state.value.shown
        quickLook.updateOrder(order = listOf(second, third))
        quickLook.next()

        assertEquals(expected = first, actual = reordered)
        assertEquals(expected = first, actual = quickLook.state.value.shown)
    }

    @Test
    fun `Esc closes the quick look once and then says there was nothing to close`() {
        val quickLook = QuickLookController()
        quickLook.open(documentId = first, order = order)

        val closed = quickLook.close()
        val closedAgain = quickLook.close()

        assertTrue(actual = closed)
        assertFalse(actual = closedAgain)
        assertFalse(actual = quickLook.isOpen)
    }

    @Test
    fun `moving to another document ends an edit in place`() {
        val quickLook = QuickLookController()
        quickLook.open(documentId = first, order = order)
        quickLook.edit(fact = QuickFact.DATE)
        val editing = quickLook.state.value.editing

        quickLook.next()

        assertEquals(expected = QuickFact.DATE, actual = editing)
        assertNull(actual = quickLook.state.value.editing)
    }
}
