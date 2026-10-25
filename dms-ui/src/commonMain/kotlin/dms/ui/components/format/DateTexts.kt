package dms.ui.components.format

import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.temporal.ChronoUnit
import kotlin.math.abs

/**
 * How every screen writes dates: exact dates as `YYYY-MM-DD`, the same form the user types, and
 * relative ones in words ("today", "3 days ago", "in 2 weeks") with the exact date beside them in a
 * tooltip.
 */
object DateTexts {

    private const val DAYS_PER_WEEK = 7L
    private const val DAYS_PER_MONTH = 30L
    private const val DAYS_PER_YEAR = 365L

    fun absolute(date: LocalDate): String {
        return date.toString()
    }

    /** The day of [instant] in [zone]. */
    fun absolute(instant: Instant, zone: ZoneId): String {
        return absolute(date = instant.atZone(zone).toLocalDate())
    }

    /** [instant] relative to [now], by calendar days in [zone]. */
    fun relative(instant: Instant, now: Instant, zone: ZoneId): String {
        return relative(date = instant.atZone(zone).toLocalDate(), today = now.atZone(zone).toLocalDate())
    }

    /** [date] relative to [today]: "today", "yesterday", "tomorrow", "N days ago", "in N weeks", … */
    fun relative(date: LocalDate, today: LocalDate): String {
        val days = ChronoUnit.DAYS.between(today, date)
        return when (days) {
            0L -> "today"
            -1L -> "yesterday"
            1L -> "tomorrow"
            else -> {
                val amount = amountOf(days = abs(days))
                if (days < 0) "$amount ago" else "in $amount"
            }
        }
    }

    private fun amountOf(days: Long): String {
        return when {
            days < DAYS_PER_WEEK * 2 -> plural(count = days, unit = "day")
            days < DAYS_PER_MONTH * 2 -> plural(count = days / DAYS_PER_WEEK, unit = "week")
            days < DAYS_PER_YEAR -> plural(count = days / DAYS_PER_MONTH, unit = "month")
            else -> plural(count = days / DAYS_PER_YEAR, unit = "year")
        }
    }

    private fun plural(count: Long, unit: String): String {
        return if (count == 1L) "1 $unit" else "$count ${unit}s"
    }
}
