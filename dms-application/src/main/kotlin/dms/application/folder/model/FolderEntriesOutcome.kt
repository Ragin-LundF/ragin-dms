package dms.application.folder.model

import dms.application.port.storage.model.FolderEntriesModel
import dms.application.port.storage.model.StorageResult

sealed interface FolderEntriesOutcome {

    /** What lies below the folder in every reachable library folder together. */
    data class Read(val entries: FolderEntriesModel) : FolderEntriesOutcome

    data object LibraryNotOpen : FolderEntriesOutcome

    /** The primary folder could not be read. */
    data class StorageFailed(val failure: StorageResult.Failure) : FolderEntriesOutcome
}
