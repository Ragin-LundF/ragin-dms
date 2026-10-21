package dms.application.fake.storage

import dms.application.port.storage.OperationJournal
import dms.application.port.storage.model.PendingOperation

class InMemoryJournal : OperationJournal {

    private val operations = mutableListOf<PendingOperation>()

    override suspend fun record(operation: PendingOperation) {
        operations += operation
    }

    override suspend fun pending(): List<PendingOperation> {
        return operations.toList()
    }

    override suspend fun remove(id: String) {
        operations.removeAll { it.id == id }
    }
}
