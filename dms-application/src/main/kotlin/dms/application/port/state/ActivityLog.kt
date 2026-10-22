package dms.application.port.state

import dms.application.port.state.model.ActivityEntryModel

/** What the tool did on this device, newest first. */
interface ActivityLog {

    suspend fun record(entry: ActivityEntryModel)

    suspend fun recent(limit: Int): List<ActivityEntryModel>
}
