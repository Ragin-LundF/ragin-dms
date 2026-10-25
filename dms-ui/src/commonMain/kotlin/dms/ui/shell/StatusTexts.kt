package dms.ui.shell

import dms.ui.components.format.DateTexts
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionStateModel
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

/** The words of the status bar: the state of the library, how many folders are online, the last scan. */
object StatusTexts {

    private val TIME: DateTimeFormatter = DateTimeFormatter.ofPattern("HH:mm")

    fun library(state: SessionStateModel, now: Instant, zone: ZoneId): String {
        val status = when (state.status) {
            LibraryStatus.Loading -> "Opening the library"
            LibraryStatus.NotConfigured -> "No library yet"
            is LibraryStatus.Open -> "Library ready"
            is LibraryStatus.PrimaryFailed, is LibraryStatus.Failed -> "Library not open"
        }
        val folders = state.folders?.folders.orEmpty().size
        val offline = state.lastScan?.failedFolders?.size ?: 0
        val reach = when {
            folders == 0 -> null
            offline > 0 -> "$folders folders, $offline offline"
            folders == 1 -> "1 folder, online"
            else -> "$folders folders, all online"
        }
        val scanned = state.lastScanAt?.let { "last scan ${scanTime(scannedAt = it, now = now, zone = zone)}" }
        return listOfNotNull(status, reach, scanned).joinToString(separator = " · ")
    }

    /** The time of day for a scan of today, otherwise how long ago. */
    private fun scanTime(scannedAt: Instant, now: Instant, zone: ZoneId): String {
        val sameDay = scannedAt.atZone(zone).toLocalDate() == now.atZone(zone).toLocalDate()
        if (sameDay) {
            return TIME.format(scannedAt.atZone(zone))
        }
        return DateTexts.relative(instant = scannedAt, now = now, zone = zone)
    }
}
