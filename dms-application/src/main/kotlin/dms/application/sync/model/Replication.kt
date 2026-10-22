package dms.application.sync.model

import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.LibraryFolderIdModel

/** The result of applying one change to the primary folder and every sync folder. */
sealed interface Replication<out T> {

    /** The primary was changed; [pendingFolders] failed and were recorded in the journal. */
    data class Done<T>(val value: T, val pendingFolders: Set<LibraryFolderIdModel>) : Replication<T>

    /** The primary could not be changed, so no folder was touched. */
    data class PrimaryFailed(val failure: StorageResult.Failure) : Replication<Nothing>
}
