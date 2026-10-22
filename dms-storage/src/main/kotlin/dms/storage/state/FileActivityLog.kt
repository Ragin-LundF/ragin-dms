package dms.storage.state

import dms.application.port.state.ActivityLog
import dms.application.port.state.model.ActivityEntryModel
import dms.application.port.state.model.ActivityKind
import dms.storage.state.dto.ActivityEntryDto
import dms.storage.state.dto.ActivityLogDto
import dms.storage.yaml.YamlFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.nio.file.Path
import java.time.Instant

/** The last [capacity] activity entries in `activity.yaml`, newest first. */
class FileActivityLog(
    path: Path,
    private val capacity: Int = DEFAULT_CAPACITY,
) : ActivityLog {

    private val file = YamlFile(path = path, type = ActivityLogDto::class.java)
    private val mutex = Mutex()

    override suspend fun record(entry: ActivityEntryModel) {
        mutex.withLock {
            withContext(context = Dispatchers.IO) {
                val dto = ActivityEntryDto(at = entry.at.toString(), kind = entry.kind.name, subject = entry.subject)
                val entries = listOf(dto) + file.read()?.entries.orEmpty()
                file.write(value = ActivityLogDto(entries = entries.take(n = capacity)))
            }
        }
    }

    override suspend fun recent(limit: Int): List<ActivityEntryModel> {
        return mutex.withLock {
            withContext(context = Dispatchers.IO) {
                file.read()?.entries.orEmpty()
                    .mapNotNull { toDomain(dto = it) }
                    .take(n = limit)
            }
        }
    }

    /** Entries of a kind this version does not know are skipped. */
    private fun toDomain(dto: ActivityEntryDto): ActivityEntryModel? {
        val kind = ActivityKind.entries.firstOrNull { it.name == dto.kind } ?: return null
        return ActivityEntryModel(at = Instant.parse(dto.at), kind = kind, subject = dto.subject)
    }

    private companion object {
        const val DEFAULT_CAPACITY = 200
    }
}
