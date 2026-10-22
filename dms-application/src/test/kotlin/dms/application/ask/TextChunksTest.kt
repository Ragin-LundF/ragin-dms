package dms.application.ask

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class TextChunksTest {

    @Test
    fun `a short text is one passage and an empty text one empty passage`() {
        assertEquals(expected = listOf("Sample bill"), actual = TextChunks.of(text = "  Sample bill ").map { it.text })
        assertEquals(expected = listOf(""), actual = TextChunks.of(text = " ").map { it.text })
    }

    @Test
    fun `a long text is cut at blanks into overlapping passages that cover all of it`() {
        val text = (1..2_000).joinToString(separator = " ") { "word$it" }

        val chunks = TextChunks.of(text = text)

        assertTrue(actual = chunks.size > 1)
        assertTrue(actual = chunks.all { it.text.length <= TextChunks.SIZE })
        assertEquals(expected = 0, actual = chunks.first().start)
        assertEquals(expected = text.length, actual = chunks.last().end)
        chunks.zipWithNext().forEach { (first, second) ->
            assertTrue(actual = second.start < first.end, message = "passages overlap")
            assertTrue(actual = text[first.end] == ' ' || first.end == text.length, message = "cut at a blank")
        }
    }

    @Test
    fun `a huge text is capped`() {
        val text = "x".repeat(n = TextChunks.SIZE * (TextChunks.MAX_CHUNKS + 10))

        assertEquals(expected = TextChunks.MAX_CHUNKS, actual = TextChunks.of(text = text).size)
    }
}
