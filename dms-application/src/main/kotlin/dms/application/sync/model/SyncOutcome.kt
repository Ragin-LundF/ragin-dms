package dms.application.sync.model

import dms.application.port.storage.model.StorageResult

sealed interface SyncOutcome {

    data class Completed(val report: SyncReport) : SyncOutcome

    data object LibraryNotOpen : SyncOutcome

    /** The primary folder could not be scanned or written; nothing was copied. */
    data class PrimaryFailed(val failure: StorageResult.Failure) : SyncOutcome
}
