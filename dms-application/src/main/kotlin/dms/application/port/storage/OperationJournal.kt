package dms.application.port.storage

import dms.application.port.storage.model.PendingOperation

/** Operations that could not be applied to a sync folder, kept locally until they are retried. */
interface OperationJournal {

    suspend fun record(operation: PendingOperation)

    suspend fun pending(): List<PendingOperation>

    suspend fun remove(id: String)
}
