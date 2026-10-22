package dms.application.library.model

import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.CatalogMergeConflict
import dms.domain.catalog.model.CatalogModel
import dms.domain.library.model.LibraryFolderIdModel

sealed interface OpenLibraryOutcome {

    /**
     * The library is open. [mergeConflicts] are decisions the merge of the catalog copies made;
     * [unreachableFolders] could not be read; [foreignFolders] hold the catalog of another library and
     * were ignored; [pendingFolders] did not receive the merged catalog yet.
     */
    data class Opened(
        val catalog: CatalogModel,
        val mergeConflicts: List<CatalogMergeConflict>,
        val unreachableFolders: Set<LibraryFolderIdModel>,
        val foreignFolders: Set<LibraryFolderIdModel>,
        val pendingFolders: Set<LibraryFolderIdModel>,
    ) : OpenLibraryOutcome

    data object NotConfigured : OpenLibraryOutcome

    /** The primary folder or its catalog could not be read; nothing was opened. */
    data class PrimaryFailed(val failure: StorageResult.Failure) : OpenLibraryOutcome
}
