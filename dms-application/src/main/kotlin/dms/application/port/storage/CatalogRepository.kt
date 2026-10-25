package dms.application.port.storage

import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.library.model.LibraryFolderModel

/** Reads and writes the catalog file of one library folder. */
interface CatalogRepository {

    suspend fun load(folder: LibraryFolderModel): CatalogLoad

    /**
     * Writes [catalog], unless the file on disk holds a higher revision than [basedOn], the newest
     * revision the caller has seen: then nothing is written and the result is
     * [dms.application.port.storage.model.StorageFailure.CATALOG_CHANGED], so a change another device
     * saved in the meantime is never overwritten.
     */
    suspend fun save(
        folder: LibraryFolderModel,
        catalog: CatalogModel,
        basedOn: CatalogRevisionModel,
    ): StorageResult<Unit>

    /** Moves the conflict copies of the catalog file to the trash, after they were merged and saved. */
    suspend fun discardConflictCopies(folder: LibraryFolderModel): StorageResult<Unit>
}
