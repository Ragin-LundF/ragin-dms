package dms.ui.components.format

import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import kotlin.test.Test
import kotlin.test.assertEquals

class DateTextsTest {

    private val today = LocalDate.of(2026, 10, 25)

    @Test
    fun `near days have their own words`() {
        assertEquals(expected = "today", actual = DateTexts.relative(date = today, today = today))
        assertEquals(expected = "yesterday", actual = DateTexts.relative(date = today.minusDays(1), today = today))
        assertEquals(expected = "tomorrow", actual = DateTexts.relative(date = today.plusDays(1), today = today))
    }

    @Test
    fun `further days are counted in days, weeks, months and years, back and ahead`() {
        val cases = mapOf(
            today.minusDays(3) to "3 days ago",
            today.minusDays(13) to "13 days ago",
            today.minusDays(14) to "2 weeks ago",
            today.plusDays(21) to "in 3 weeks",
            today.minusDays(60) to "2 months ago",
            today.minusDays(400) to "1 year ago",
            today.plusDays(800) to "in 2 years",
        )
        cases.forEach { (date, text) ->
            assertEquals(expected = text, actual = DateTexts.relative(date = date, today = today), message = text)
        }
    }

    @Test
    fun `an instant counts by the calendar day of the zone`() {
        val berlin = ZoneId.of("Europe/Berlin")
        val lateEvening = Instant.parse("2026-10-24T23:30:00Z")
        val now = Instant.parse("2026-10-25T08:00:00Z")

        assertEquals(expected = "today", actual = DateTexts.relative(instant = lateEvening, now = now, zone = berlin))
        assertEquals(expected = "2026-10-25", actual = DateTexts.absolute(instant = lateEvening, zone = berlin))
    }
}
