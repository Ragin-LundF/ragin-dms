package dms.application.port.storage

import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.Catalog
import dms.domain.change.FolderSnapshot
import dms.domain.library.LibraryFolder

/**
 * Lists every file of one library folder. Files the [catalog] does not know are always hashed; known
 * files only when they changed since the last scan (see `FileFacts`).
 */
interface LibraryScanner {

    suspend fun scan(folder: LibraryFolder, catalog: Catalog): StorageResult<FolderSnapshot>
}
