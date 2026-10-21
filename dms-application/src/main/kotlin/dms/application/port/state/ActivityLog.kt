package dms.application.port.state

import dms.application.port.state.model.ActivityEntry

/** What the tool did on this device, newest first. */
interface ActivityLog {

    suspend fun record(entry: ActivityEntry)

    suspend fun recent(limit: Int): List<ActivityEntry>
}
