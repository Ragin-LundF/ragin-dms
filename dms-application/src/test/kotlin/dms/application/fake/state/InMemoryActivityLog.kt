package dms.application.fake.state

import dms.application.port.state.ActivityLog
import dms.application.port.state.model.ActivityEntryModel

class InMemoryActivityLog : ActivityLog {

    private val entries = mutableListOf<ActivityEntryModel>()

    override suspend fun record(entry: ActivityEntryModel) {
        entries.add(index = 0, element = entry)
    }

    override suspend fun recent(limit: Int): List<ActivityEntryModel> {
        return entries.take(n = limit)
    }
}
