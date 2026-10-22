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

    @Test
    fun `the outline lists folders with counts, categories and samples, or asks for no folder`() {
        val withOutline = PromptTexts.user(request = Requests.request(outline = Requests.outline), text = "x")
        val without = PromptTexts.user(request = Requests.request(), text = "x")

        kotlin.test.assertContains(
            charSequence = withOutline,
            other = "Home | 3 documents | mostly Invoices / Energy | e.g. Sample bill",
        )
        kotlin.test.assertContains(
            charSequence = withOutline,
            other = "Home/Power | 2 documents | mostly Invoices / Energy | 4 more subfolders",
        )
        kotlin.test.assertContains(charSequence = without, other = "answer targetFolder with null")
    }

    @Test
    fun `the system prompt lists the custom fields but never a retention`() {
        val system = PromptTexts.system(
            request = Requests.request(fieldDefinitions = dms.domain.field.FieldPresets.ALL),
        )

        kotlin.test.assertContains(
            charSequence = system,
            other = "amount (Amount, MONEY in EUR unless stated otherwise)",
        )
        kotlin.test.assertContains(charSequence = system, other = "cancel-by (Cancel by, DATE)")
        kotlin.test.assertFalse(actual = system.contains(other = "retention", ignoreCase = true))
    }

    @Test
    fun `values the rules decided are named as fixed, and nothing is said without rules`() {
        val fixed = dms.domain.rule.match.model.RuleResultModel(
            correspondent = "Sample Energy",
            categoryId = Requests.energy.id,
            tags = setOf(dms.domain.taxonomy.model.TagNameModel.of(raw = "Tax")),
        )

        val section = PromptTexts.rulesSection(request = Requests.request().copy(fixedValues = fixed))

        kotlin.test.assertEquals(
            expected = "Already decided by the person's rules, keep these values: category: Invoices / Energy; " +
                "correspondent: Sample Energy; tags: Tax",
            actual = section,
        )
        kotlin.test.assertEquals(expected = "", actual = PromptTexts.rulesSection(request = Requests.request()))
    }
}
