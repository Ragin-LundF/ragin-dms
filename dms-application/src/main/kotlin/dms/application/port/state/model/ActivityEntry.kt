package dms.application.port.state.model

import java.time.Instant

/** One line of the activity log. [subject] names what was affected, usually a path. */
data class ActivityEntry(
    val at: Instant,
    val kind: ActivityKind,
    val subject: String,
)
