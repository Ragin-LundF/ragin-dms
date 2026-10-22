package dms.ai.assistant

import dms.domain.taxonomy.model.TagNameModel
import kotlin.test.Test
import kotlin.test.assertEquals

class TagAnswerParserTest {

    private val existing = setOf(TagNameModel.of(raw = "Steuer"), TagNameModel.of(raw = "Strom"))

    @Test
    fun `a plain comma-separated answer is taken as it is`() {
        val tags = TagAnswerParser.parse(answer = "Rechnung, Strom, Steuer", existingTags = existing)

        assertEquals(expected = listOf("Rechnung", "Strom", "Steuer"), actual = tags.map { it.value })
    }

    @Test
    fun `lead-ins, bullets, numbering, quotes and fences are removed`() {
        val answer = """
            Here are the tags:
            ```
            1. "rechnung"
            - #strom
            * steuer.
            ```
        """.trimIndent()

        val tags = TagAnswerParser.parse(answer = answer, existingTags = existing)

        assertEquals(expected = listOf("rechnung", "Strom", "Steuer"), actual = tags.map { it.value })
    }

    @Test
    fun `sentences are no tags, and duplicates are dropped`() {
        val answer = "Tags: Rechnung; rechnung, This document is clearly an electricity bill"

        val tags = TagAnswerParser.parse(answer = answer, existingTags = existing)

        assertEquals(expected = listOf("Rechnung"), actual = tags.map { it.value })
    }
}
