package dms.domain.taxonomy

import dms.domain.taxonomy.model.TagNameModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals

class TagNameTest {

    @Test
    fun `whitespace is trimmed and collapsed`() {
        assertEquals(expected = "Home Office", actual = TagNameModel.of(raw = "  Home \t  Office ").value)
    }

    @Test
    fun `tags are equal regardless of case but keep their spelling`() {
        val upper = TagNameModel.of(raw = "Tax")
        val lower = TagNameModel.of(raw = "tax")

        assertEquals(expected = upper, actual = lower)
        assertEquals(expected = upper.hashCode(), actual = lower.hashCode())
        assertEquals(expected = "Tax", actual = upper.value)
        assertEquals(expected = 1, actual = setOf(upper, lower).size)
        assertNotEquals(illegal = upper, actual = TagNameModel.of(raw = "Taxes"))
    }

    @Test
    fun `blank and overlong tags are rejected`() {
        assertFailsWith<IllegalArgumentException> { TagNameModel.of(raw = "   ") }
        assertFailsWith<IllegalArgumentException> { TagNameModel.of(raw = "x".repeat(n = TagNameModel.MAX_LENGTH + 1)) }
    }
}
