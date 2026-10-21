package dms.ai.prompt

import kotlin.test.Test
import kotlin.test.assertEquals

class NamingConventionTest {

    @Test
    fun `a date prefix and a separator are recognized`() {
        val description = NamingConvention.describe(
            examples = listOf("2024-01-02 Sample - One.pdf", "2024-02-03 Sample - Two.pdf", "2024-03-04 X - Y.pdf"),
        )

        assertEquals(
            expected = "Most file names start with the document date as YYYY-MM-DD " +
                "and separate their parts with \" - \".",
            actual = description,
        )
    }

    @Test
    fun `too few examples give no convention`() {
        assertEquals(
            expected = "The library has too few files to show a convention. Use a short, descriptive name.",
            actual = NamingConvention.describe(examples = listOf("a.pdf")),
        )
    }
}
