package dms.application.fake.state

import dms.application.port.state.ActivityLog
import dms.application.port.state.model.ActivityEntry

class InMemoryActivityLog : ActivityLog {

    private val entries = mutableListOf<ActivityEntry>()

    override suspend fun record(entry: ActivityEntry) {
        entries.add(index = 0, element = entry)
    }

    override suspend fun recent(limit: Int): List<ActivityEntry> {
        return entries.take(n = limit)
    }
}
