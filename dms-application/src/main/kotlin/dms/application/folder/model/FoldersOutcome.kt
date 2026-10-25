package dms.application.folder.model

import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.RelativePathModel

sealed interface FoldersOutcome {

    /** Every folder of the primary library folder, also empty ones. */
    data class Read(val folders: Set<RelativePathModel>) : FoldersOutcome

    data object LibraryNotOpen : FoldersOutcome

    /** The primary folder could not be read. */
    data class StorageFailed(val failure: StorageResult.Failure) : FoldersOutcome
}
