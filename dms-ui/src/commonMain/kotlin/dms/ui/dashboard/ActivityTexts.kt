package dms.ui.dashboard

import dms.application.port.state.model.ActivityEntryModel
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

    fun line(entry: ActivityEntryModel): String {
        return "${time(instant = entry.at)}  ${verb(kind = entry.kind)}  ${entry.subject}"
    }

    fun verb(kind: ActivityKind): String {
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
            ActivityKind.DOCUMENT_FILED,
            ActivityKind.RESTRUCTURED,
            ActivityKind.EXPORTED,
            ActivityKind.PDF_EDITED,
            -> organizeVerb(kind = kind)
        }
    }

    /** The activities of filing, restructuring, exporting and editing PDFs. */
    private fun organizeVerb(kind: ActivityKind): String {
        return when (kind) {
            ActivityKind.DOCUMENT_FILED -> "Filed"
            ActivityKind.RESTRUCTURED -> "Restructured"
            ActivityKind.EXPORTED -> "Exported"
            else -> "Edited PDF"
        }
    }
}
