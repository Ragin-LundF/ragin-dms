package dms.application.common

import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRejection
import dms.domain.library.FileNameViolation
import dms.domain.library.LibraryFolderId

/** The outcome shared by the use cases that change files or the catalog. */
sealed interface OperationOutcome {

    /** Adds [folders] to the pending folders of a [Done] outcome; every other outcome stays as it is. */
    fun withPendingFolders(folders: Set<LibraryFolderId>): OperationOutcome {
        if (this is Done) {
            return copy(pendingFolders = pendingFolders + folders)
        }
        return this
    }

    /**
     * The change is in the primary folder and the catalog. [pendingFolders] are sync folders that
     * could not be updated yet; their operations wait in the journal.
     */
    data class Done(val catalog: Catalog, val pendingFolders: Set<LibraryFolderId>) : OperationOutcome

    /** No library is configured or opened yet. */
    data object LibraryNotOpen : OperationOutcome

    data object NotFound : OperationOutcome

    /** The folders no longer look the way the task describes; scan again and decide anew. */
    data object ScanOutdated : OperationOutcome

    /** A destructive operation was requested without the user's confirmation. */
    data object NotConfirmed : OperationOutcome

    data class InvalidName(val violation: FileNameViolation) : OperationOutcome

    /** Another entry in the target folder already has this name (ignoring case). */
    data object NameTaken : OperationOutcome

    data class Rejected(val reason: CatalogRejection) : OperationOutcome

    /** The primary folder could not be changed; nothing was changed anywhere. */
    data class StorageFailed(val failure: StorageResult.Failure) : OperationOutcome
}
