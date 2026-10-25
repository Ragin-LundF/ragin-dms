package dms.ui.session

import java.time.LocalDate
import java.time.format.DateTimeParseException

/** Turns a date the user typed (`YYYY-MM-DD`) into a date, with one message for every date field. */
object DateInput {

    const val PROBLEM: String = "Write the date as YYYY-MM-DD, such as 2030-12-31"

    /** The date in [raw], or `null` if it is not one; surrounding spaces do not count. */
    fun parse(raw: String): LocalDate? {
        return try {
            LocalDate.parse(raw.trim())
        } catch (ignored: DateTimeParseException) {
            null
        }
    }
}
