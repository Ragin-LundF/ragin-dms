package dms.application.port.storage

import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.library.LibraryFolder

/** Reads and writes the catalog file of one library folder. */
interface CatalogRepository {

    suspend fun load(folder: LibraryFolder): CatalogLoad

    suspend fun save(folder: LibraryFolder, catalog: Catalog): StorageResult<Unit>

    /** Moves the conflict copies of the catalog file to the trash, after they were merged and saved. */
    suspend fun discardConflictCopies(folder: LibraryFolder): StorageResult<Unit>
}
