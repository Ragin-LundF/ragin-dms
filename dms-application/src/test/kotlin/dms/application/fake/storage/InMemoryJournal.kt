package dms.application.fake.storage

import dms.application.port.storage.OperationJournal
import dms.application.port.storage.model.PendingOperationModel

class InMemoryJournal : OperationJournal {

    private val operations = mutableListOf<PendingOperationModel>()

    override suspend fun record(operation: PendingOperationModel) {
        operations += operation
    }

    override suspend fun pending(): List<PendingOperationModel> {
        return operations.toList()
    }

    override suspend fun remove(id: String) {
        operations.removeAll { it.id == id }
    }
}
