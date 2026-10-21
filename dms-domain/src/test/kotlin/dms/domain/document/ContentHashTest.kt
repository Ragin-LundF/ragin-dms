package dms.domain.document

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class ContentHashTest {

    @Test
    fun `upper case hex is normalized to lower case`() {
        val hash = ContentHash.fromHex(hex = "AB".repeat(n = 32))

        assertEquals(expected = "ab".repeat(n = 32), actual = hash.value)
    }

    @Test
    fun `anything but 64 hex characters is rejected`() {
        listOf("", "abc", "g".repeat(n = 64), "a".repeat(n = 65)).forEach { value ->
            assertFailsWith<IllegalArgumentException>(message = value) { ContentHash(value = value) }
        }
    }
}
