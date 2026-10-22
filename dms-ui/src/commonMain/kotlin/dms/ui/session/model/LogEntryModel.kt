package dms.ui.session.model

import java.time.Instant

/** One line of the session log. */
data class LogEntryModel(
    val at: Instant,
    val text: String,
    val isError: Boolean = false,
)
