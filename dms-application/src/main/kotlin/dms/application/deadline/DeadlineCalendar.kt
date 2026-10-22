package dms.application.deadline

import dms.domain.catalog.model.CatalogModel
import dms.domain.deadline.DeadlinePlanner
import dms.domain.deadline.model.UpcomingDateModel
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.util.Locale

/**
 * The upcoming deadlines and retention dates as an iCalendar file (`.ics`), one all-day event per
 * date, so the user can import them into a calendar application. Pure; writing the file is the UI's.
 */
object DeadlineCalendar {

    private const val LIMIT = 500
    private val DAY: DateTimeFormatter = DateTimeFormatter.ofPattern("yyyyMMdd", Locale.ROOT)

    fun of(catalog: CatalogModel, today: LocalDate): String {
        val events = DeadlinePlanner.upcoming(catalog = catalog, today = today, limit = LIMIT)
            .flatMap { event(catalog = catalog, upcoming = it, today = today) }
        val header = listOf("BEGIN:VCALENDAR", "VERSION:2.0", "PRODID:-//ragin-dms//deadlines//EN")
        return (header + events + "END:VCALENDAR").joinToString(separator = "\r\n", postfix = "\r\n")
    }

    private fun event(catalog: CatalogModel, upcoming: UpcomingDateModel, today: LocalDate): List<String> {
        val document = catalog.documents[upcoming.documentId]
        val name = document?.metadata?.title ?: document?.path?.fileName ?: upcoming.documentId.value
        val what = upcoming.field?.let { key -> catalog.settings.field(key = key)?.label ?: key.value }
            ?: "Retention ends"
        val day = DAY.format(upcoming.date)
        return listOf(
            "BEGIN:VEVENT",
            "UID:${upcoming.documentId}-${upcoming.field?.value ?: "retention"}-$day@ragin-dms",
            "DTSTAMP:${DAY.format(today)}T000000Z",
            "DTSTART;VALUE=DATE:$day",
            "SUMMARY:${escaped(text = "$what: $name")}",
            "DESCRIPTION:${escaped(text = document?.path?.value.orEmpty())}",
            "END:VEVENT",
        )
    }

    /** RFC 5545 text: backslash, semicolon, comma and line breaks escaped. */
    private fun escaped(text: String): String {
        return text.replace(oldValue = "\\", newValue = "\\\\")
            .replace(oldValue = ";", newValue = "\\;")
            .replace(oldValue = ",", newValue = "\\,")
            .replace(oldValue = "\n", newValue = "\\n")
    }
}
