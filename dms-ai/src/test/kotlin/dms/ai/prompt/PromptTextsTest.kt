package dms.ai.prompt

import dms.ai.support.Requests
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class PromptTextsTest {

    @Test
    fun `the system prompt is filled from its template with the taxonomy`() {
        val system = PromptTexts.system(request = Requests.request())

        assertFalse(actual = "{{" in system)
        assertTrue(actual = "Existing categories: Invoices; Invoices / Energy" in system)
        assertTrue(actual = "\".pdf\"" in system)
        assertTrue(actual = "\"Tax\"" in system)
        assertTrue(actual = "Today is 2026-01-15." in system)
    }

    @Test
    fun `the user prompt carries the file name and the cut text`() {
        val text = "x".repeat(n = PromptTexts.MAX_TEXT_CHARACTERS + 50)

        val user = PromptTexts.user(request = Requests.request(), text = text)

        assertFalse(actual = "{{" in user)
        assertTrue(actual = user.startsWith(prefix = "Current file name: scan-001.pdf"))
        assertFalse(actual = "x".repeat(n = PromptTexts.MAX_TEXT_CHARACTERS + 1) in user)
    }
}
