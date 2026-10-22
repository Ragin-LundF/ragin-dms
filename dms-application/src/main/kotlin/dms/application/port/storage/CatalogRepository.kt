package dms.application.port.storage

import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.model.CatalogModel
import dms.domain.library.model.LibraryFolderModel

/** Reads and writes the catalog file of one library folder. */
interface CatalogRepository {

    suspend fun load(folder: LibraryFolderModel): CatalogLoad

    suspend fun save(folder: LibraryFolderModel, catalog: CatalogModel): StorageResult<Unit>

    /** Moves the conflict copies of the catalog file to the trash, after they were merged and saved. */
    suspend fun discardConflictCopies(folder: LibraryFolderModel): StorageResult<Unit>
}
