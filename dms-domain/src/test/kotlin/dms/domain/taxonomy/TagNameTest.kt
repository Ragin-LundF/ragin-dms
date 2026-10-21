package dms.domain.taxonomy

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals

class TagNameTest {

    @Test
    fun `whitespace is trimmed and collapsed`() {
        assertEquals(expected = "Home Office", actual = TagName.of(raw = "  Home \t  Office ").value)
    }

    @Test
    fun `tags are equal regardless of case but keep their spelling`() {
        val upper = TagName.of(raw = "Tax")
        val lower = TagName.of(raw = "tax")

        assertEquals(expected = upper, actual = lower)
        assertEquals(expected = upper.hashCode(), actual = lower.hashCode())
        assertEquals(expected = "Tax", actual = upper.value)
        assertEquals(expected = 1, actual = setOf(upper, lower).size)
        assertNotEquals(illegal = upper, actual = TagName.of(raw = "Taxes"))
    }

    @Test
    fun `blank and overlong tags are rejected`() {
        assertFailsWith<IllegalArgumentException> { TagName.of(raw = "   ") }
        assertFailsWith<IllegalArgumentException> { TagName.of(raw = "x".repeat(n = TagName.MAX_LENGTH + 1)) }
    }
}
