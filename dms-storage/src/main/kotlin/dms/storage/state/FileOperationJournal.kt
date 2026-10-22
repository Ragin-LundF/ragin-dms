package dms.storage.state

import dms.application.port.storage.OperationJournal
import dms.application.port.storage.model.PendingOperationModel
import dms.storage.state.dto.JournalDto
import dms.storage.yaml.YamlFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import java.nio.file.Path

/** The failed-operation journal in `journal.yaml`. Every change is written before the call returns. */
class FileOperationJournal(path: Path) : OperationJournal {

    private val file = YamlFile(path = path, type = JournalDto::class.java)
    private val mutex = Mutex()

    override suspend fun record(operation: PendingOperationModel) {
        update { it + operation }
    }

    override suspend fun pending(): List<PendingOperationModel> {
        return mutex.withLock { read() }
    }

    override suspend fun remove(id: String) {
        update { operations -> operations.filterNot { it.id == id } }
    }

    private suspend fun update(change: (List<PendingOperationModel>) -> List<PendingOperationModel>) {
        mutex.withLock {
            val operations = change(read())
            withContext(context = Dispatchers.IO) {
                file.write(value = JournalDto(operations = operations.map { OperationMapper.toDto(operation = it) }))
            }
        }
    }

    private suspend fun read(): List<PendingOperationModel> {
        return withContext(context = Dispatchers.IO) {
            file.read()?.operations.orEmpty().mapNotNull { OperationMapper.toDomain(dto = it) }
        }
    }
}
