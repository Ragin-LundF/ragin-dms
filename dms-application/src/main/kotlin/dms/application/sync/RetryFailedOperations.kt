package dms.application.sync

import dms.application.library.CatalogStore
import dms.application.port.storage.OperationJournal
import dms.application.port.storage.model.PendingOperationModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.application.sync.model.RetryReportModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFoldersModel

/**
 * Replays the journal: the operations of each sync folder in the order they were recorded. The first
 * operation that still fails stops that folder, so later operations never overtake earlier ones.
 * Operations that no longer apply are dropped.
 */
class RetryFailedOperations(
    private val store: CatalogStore,
    private val journal: OperationJournal,
    private val replicator: Replicator,
) {

    /** Returns `null` while no library is open. */
    suspend operator fun invoke(): RetryReportModel? {
        val library = store.current() ?: return null
        return journal.pending()
            .sortedBy { it.recordedAt }
            .groupBy { it.folderId }
            .map { (folderId, operations) ->
                retry(folders = library.folders, folderId = folderId, operations = operations)
            }
            .fold(initial = RetryReportModel()) { total, report -> total + report }
    }

    private suspend fun retry(
        folders: LibraryFoldersModel,
        folderId: LibraryFolderIdModel,
        operations: List<PendingOperationModel>,
    ): RetryReportModel {
        val target = folders.syncFolders.firstOrNull { it.id == folderId }
        if (target == null) {
            // The folder was removed or became the primary; its operations no longer apply.
            operations.forEach { journal.remove(id = it.id) }
            return RetryReportModel(dropped = operations.size)
        }
        var report = RetryReportModel()
        for ((index, pending) in operations.withIndex()) {
            val result = replicator.execute(primary = folders.primary, target = target, operation = pending.operation)
            val step = outcome(pending = pending, result = result)
                ?: return report + RetryReportModel(stillPending = operations.size - index)
            report += step
        }
        return report
    }

    /** Removes a finished or pointless operation from the journal; `null` if it must stay. */
    private suspend fun outcome(pending: PendingOperationModel, result: StorageResult<Unit>): RetryReportModel? {
        return when {
            result is StorageResult.Success -> {
                journal.remove(id = pending.id)
                RetryReportModel(succeeded = 1)
            }

            (result as StorageResult.Failure).reason in OBSOLETE -> {
                journal.remove(id = pending.id)
                RetryReportModel(dropped = 1)
            }

            else -> null
        }
    }

    private companion object {
        /**
         * The source is gone, or the target already holds something else at that path: the operation
         * no longer applies, and the next scan shows any difference that is left.
         */
        val OBSOLETE = setOf(StorageFailure.NOT_FOUND, StorageFailure.ALREADY_EXISTS)
    }
}
