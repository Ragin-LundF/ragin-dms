package dms.ui.dashboard

import dms.application.port.state.model.ActivityEntry
import dms.application.port.state.model.ActivityKind
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

/** One line per activity entry, in the local time of this device. */
object ActivityTexts {

    private val TIME = DateTimeFormatter.ofPattern("dd.MM. HH:mm").withZone(ZoneId.systemDefault())

    fun time(instant: Instant): String {
        return TIME.format(instant)
    }

    fun line(entry: ActivityEntry): String {
        return "${time(instant = entry.at)}  ${verb(kind = entry.kind)}  ${entry.subject}"
    }

    private fun verb(kind: ActivityKind): String {
        return when (kind) {
            ActivityKind.DOCUMENT_ADDED -> "Added"
            ActivityKind.DOCUMENT_MOVED -> "Moved"
            ActivityKind.DOCUMENT_TRASHED -> "Trashed"
            ActivityKind.DOCUMENT_RESTORED -> "Restored"
            ActivityKind.FOLDER_CREATED -> "Created folder"
            ActivityKind.FOLDER_MOVED -> "Moved folder"
            ActivityKind.CATALOG_EDITED -> "Edited catalog"
            ActivityKind.SUGGESTION_APPROVED -> "Approved suggestion"
            ActivityKind.CONFLICT_RESOLVED -> "Resolved"
            ActivityKind.SYNCHRONIZED -> "Synchronized"
        }
    }
}
