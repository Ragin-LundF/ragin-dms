package dms.domain.library

import java.text.Normalizer
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class FileNamePolicyTest {

    @Test
    fun `ordinary names with spaces umlauts and dates are valid`() {
        listOf("2024-01-15 Example Invoice.pdf", "Übersicht März.docx", "notes", "a.b.c").forEach { name ->
            assertNull(actual = FileNamePolicy.check(name = name), message = name)
        }
    }

    @Test
    fun `each rule reports its own violation`() {
        val expectations = mapOf(
            "" to FileNameViolation.EMPTY,
            "x".repeat(n = FileNamePolicy.MAX_LENGTH + 1) to FileNameViolation.TOO_LONG,
            "tab\there" to FileNameViolation.CONTROL_CHARACTER,
            "a:b.pdf" to FileNameViolation.FORBIDDEN_CHARACTER,
            "a/b.pdf" to FileNameViolation.FORBIDDEN_CHARACTER,
            "what?.pdf" to FileNameViolation.FORBIDDEN_CHARACTER,
            ".hidden" to FileNameViolation.LEADING_DOT_OR_SPACE,
            " lead.pdf" to FileNameViolation.LEADING_DOT_OR_SPACE,
            "trail." to FileNameViolation.TRAILING_DOT_OR_SPACE,
            "trail " to FileNameViolation.TRAILING_DOT_OR_SPACE,
            "CON" to FileNameViolation.RESERVED_NAME,
            "nul.txt" to FileNameViolation.RESERVED_NAME,
            "Com1.pdf" to FileNameViolation.RESERVED_NAME,
        )

        expectations.forEach { (name, violation) ->
            assertEquals(expected = violation, actual = FileNamePolicy.check(name = name), message = name)
        }
    }

    @Test
    fun `names that only contain a reserved word are valid`() {
        assertNull(actual = FileNamePolicy.check(name = "CONTRACT.pdf"))
        assertNull(actual = FileNamePolicy.check(name = "COM10.pdf"))
    }

    @Test
    fun `names differing only by case or normalization are the same name`() {
        val decomposed = Normalizer.normalize("Übersicht.pdf", Normalizer.Form.NFD)

        assertTrue(actual = FileNamePolicy.isSameName(first = "Report.PDF", second = "report.pdf"))
        assertTrue(actual = FileNamePolicy.isSameName(first = decomposed, second = "übersicht.pdf"))
        assertFalse(actual = FileNamePolicy.isSameName(first = "report-1.pdf", second = "report-2.pdf"))
    }

    @Test
    fun `the next free name adds a counter before the extension`() {
        val siblings = listOf("report.pdf", "Report (2).pdf", "notes")

        assertEquals(expected = "new.pdf", actual = FileNamePolicy.nextFreeName(name = "new.pdf", siblings = siblings))
        assertEquals(
            expected = "report (3).pdf",
            actual = FileNamePolicy.nextFreeName(name = "report.pdf", siblings = siblings),
        )
        assertEquals(expected = "notes (2)", actual = FileNamePolicy.nextFreeName(name = "notes", siblings = siblings))
    }

    @Test
    fun `collision check looks at all siblings`() {
        val siblings = listOf("alpha.pdf", "Beta.pdf")

        assertTrue(actual = FileNamePolicy.collidesWithAny(name = "beta.pdf", siblings = siblings))
        assertFalse(actual = FileNamePolicy.collidesWithAny(name = "gamma.pdf", siblings = siblings))
    }
}
