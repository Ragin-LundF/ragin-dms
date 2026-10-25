package dms.domain.library

import dms.domain.library.model.RelativePathModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class TemporaryNamesTest {

    @Test
    fun `a sibling lies in the same folder, starts with the prefix and keeps the extension`() {
        val sibling = TemporaryNames.siblingOf(path = RelativePathModel.of(raw = "letters/scan.pdf"), uniqueId = "42")

        assertEquals(expected = "letters/.dms-tmp-42.pdf", actual = sibling.value)
        assertTrue(actual = TemporaryNames.isTemporary(name = sibling.fileName))
    }

    @Test
    fun `a sibling of a file at the top level without an extension has none`() {
        val sibling = TemporaryNames.siblingOf(path = RelativePathModel.of(raw = "notes"), uniqueId = "7")

        assertEquals(expected = ".dms-tmp-7", actual = sibling.value)
    }

    @Test
    fun `an ordinary name is not temporary`() {
        assertFalse(actual = TemporaryNames.isTemporary(name = "dms-tmp-report.pdf"))
    }
}
