package dms.ui.system

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ErrorTextsTest {

    @Test
    fun `a class that cannot be loaded asks for a restart and names the class`() {
        val message = ErrorTexts.messageOf(throwable = NoClassDefFoundError("dms/ui/library/drag/DragPreviewKt"))

        assertTrue(actual = message.contains(other = "Restart Ragin-DMS"))
        assertTrue(actual = message.contains(other = "dms/ui/library/drag/DragPreviewKt"))
    }

    @Test
    fun `any other failure names its type and message`() {
        val message = ErrorTexts.messageOf(throwable = IllegalStateException("Sample failure"))

        assertTrue(actual = message.contains(other = "IllegalStateException: Sample failure"))
    }

    @Test
    fun `a failure without a message still names its type`() {
        val message = ErrorTexts.messageOf(throwable = IllegalStateException())

        assertEquals(
            expected = "Something unexpected went wrong (IllegalStateException). The window closes now; " +
                "open Ragin-DMS again to continue.",
            actual = message,
        )
    }
}
