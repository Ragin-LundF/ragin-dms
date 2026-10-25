package dms.ui.library.selection

import dms.domain.document.model.DocumentIdModel
import kotlin.test.Test
import kotlin.test.assertEquals

class DocumentSelectionTest {

    private val shown = listOf("a", "b", "c", "d", "e").map { DocumentIdModel(value = it) }
    private val selection = DocumentSelection(shown = { shown })

    @Test
    fun `a tick toggles one document`() {
        selection.toggle(documentId = id(value = "b"))
        selection.toggle(documentId = id(value = "d"))
        selection.toggle(documentId = id(value = "b"))

        assertEquals(expected = setOf(id(value = "d")), actual = selection.state.value.selected)
    }

    @Test
    fun `shift ticks every shown document from the last ticked one, in either direction`() {
        selection.toggle(documentId = id(value = "d"))
        selection.extend(documentId = id(value = "b"))

        assertEquals(
            expected = setOf("b", "c", "d").map { id(value = it) }.toSet(),
            actual = selection.state.value.selected,
        )
    }

    @Test
    fun `shift without a ticked document ticks only the clicked one`() {
        selection.extend(documentId = id(value = "c"))

        assertEquals(expected = setOf(id(value = "c")), actual = selection.state.value.selected)
    }

    @Test
    fun `select all ticks every shown document and clear unticks them`() {
        selection.selectAll()
        val all = selection.state.value.selected
        selection.clear()

        assertEquals(expected = shown.toSet(), actual = all)
        assertEquals(expected = emptySet(), actual = selection.state.value.selected)
    }

    private fun id(value: String): DocumentIdModel {
        return DocumentIdModel(value = value)
    }
}
