package dms.application.pdf.rebuild

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LineRepairTest {

    @Test
    fun `NUL and the replacement character count as lost`() {
        assertTrue(actual = LineRepair.hasLostCharacters(text = "Gr\u0000\u0000e"))
        assertTrue(actual = LineRepair.hasLostCharacters(text = "Gr�e"))
        assertFalse(actual = LineRepair.hasLostCharacters(text = "Grüße"))
    }

    @Test
    fun `lost runs at the start, in the middle and at the end are filled`() {
        assertEquals(expected = "Übung", actual = LineRepair.repaired(text = "\u0000bung", reread = "Übung"))
        assertEquals(expected = "Köln", actual = LineRepair.repaired(text = "K\u0000ln", reread = "Köln"))
        assertEquals(expected = "Gebühr", actual = LineRepair.repaired(text = "Geb\u0000hr", reread = "Gebühr"))
        assertEquals(expected = "Maß", actual = LineRepair.repaired(text = "Ma\u0000", reread = "Maß"))
    }

    @Test
    fun `several runs are filled independently`() {
        assertEquals(
            expected = "Grüße aus Köln",
            actual = LineRepair.repaired(text = "Gr\u0000\u0000e aus K\u0000ln", reread = "Grüße aus Köln"),
        )
    }

    @Test
    fun `a run of n lost characters may become fewer, but not more, and never none`() {
        assertEquals(expected = "Gräte", actual = LineRepair.repaired(text = "Gr\u0000\u0000te", reread = "Gräte"))
        assertNull(actual = LineRepair.repaired(text = "Gr\u0000te", reread = "Graete"))
        assertNull(actual = LineRepair.repaired(text = "Gr\u0000te", reread = "Grte"))
    }

    @Test
    fun `spacing of the reread does not matter, but a lost character never becomes a space`() {
        val repaired = LineRepair.repaired(text = "Gr\u0000\u0000e aus K\u0000ln", reread = "  Grüße   aus Köln ")
        assertEquals(expected = "Grüße aus Köln", actual = repaired)
        assertNull(actual = LineRepair.repaired(text = "a\u0000b", reread = "a b"))
    }

    @Test
    fun `a reread that changes a known character is refused`() {
        assertNull(actual = LineRepair.repaired(text = "K\u0000ln 50667", reread = "Köln 50668"))
        assertNull(actual = LineRepair.repaired(text = "Herr M\u0000ller", reread = "Frau Müller"))
    }

    @Test
    fun `a line lost completely takes any word of up to that many characters, only the ink check is left`() {
        assertEquals(expected = "Öl", actual = LineRepair.repaired(text = "\u0000\u0000\u0000", reread = "Öl"))
        assertNull(actual = LineRepair.repaired(text = "\u0000\u0000\u0000", reread = "Öl ab"))
    }
}
