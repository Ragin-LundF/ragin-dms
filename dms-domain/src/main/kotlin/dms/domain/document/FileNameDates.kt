package dms.domain.document

import java.time.DateTimeException
import java.time.LocalDate

/**
 * Finds the date a file name carries, the way people commonly write it: `2024-03-15`, `2024_03_15`,
 * `2024.03.15`, `20240315`, `15.03.2024`, `15-03-2024`, or `2024-03` for a month. The leftmost
 * match wins; numbers that are no valid date are ignored.
 */
object FileNameDates {

    private const val FIRST_YEAR = 1900
    private const val LAST_YEAR = 2199

    private val YEAR_FIRST = Regex(
        pattern = "(?<!\\d)(?<year>\\d{4})(?<sep>[-_.]?)(?<month>\\d{2})\\k<sep>(?<day>\\d{2})(?!\\d)",
    )
    private val DAY_FIRST = Regex(
        pattern = "(?<!\\d)(?<day>\\d{2})(?<sep>[.-])(?<month>\\d{2})\\k<sep>(?<year>\\d{4})(?!\\d)",
    )
    private val MONTH = Regex(pattern = "(?<!\\d)(\\d{4})[-_.](\\d{2})(?![-_.]?\\d)")

    fun dateIn(name: String): LocalDate? {
        val fullDates = YEAR_FIRST.findAll(input = name) + DAY_FIRST.findAll(input = name)
        val candidates = fullDates.mapNotNull { match ->
            dateOf(
                year = match.group(name = "year"),
                month = match.group(name = "month"),
                day = match.group(name = "day"),
            )?.let { match.range.first to it }
        } + MONTH.findAll(input = name).mapNotNull { match ->
            val (year, month) = match.destructured
            dateOf(year = year, month = month, day = "01")?.let { match.range.first to it }
        }
        return candidates.minByOrNull { it.first }?.second
    }

    private fun MatchResult.group(name: String): String = groups[name]?.value.orEmpty()

    private fun dateOf(year: String, month: String, day: String): LocalDate? {
        val yearNumber = year.toInt()
        if (yearNumber !in FIRST_YEAR..LAST_YEAR) {
            return null
        }
        return try {
            LocalDate.of(yearNumber, month.toInt(), day.toInt())
        } catch (ignored: DateTimeException) {
            null
        }
    }
}
