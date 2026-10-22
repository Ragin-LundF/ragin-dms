package dms.application.port.storage

import dms.application.port.storage.model.PendingOperationModel

/** Operations that could not be applied to a sync folder, kept locally until they are retried. */
interface OperationJournal {

    suspend fun record(operation: PendingOperationModel)

    suspend fun pending(): List<PendingOperationModel>

    suspend fun remove(id: String)
}
