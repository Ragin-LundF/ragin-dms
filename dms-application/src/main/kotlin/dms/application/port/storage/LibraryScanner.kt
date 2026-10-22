package dms.application.port.storage

import dms.application.port.storage.model.StorageResult
import dms.domain.catalog.model.CatalogModel
import dms.domain.change.model.FolderSnapshotModel
import dms.domain.library.model.LibraryFolderModel

/**
 * Lists every file of one library folder. Files the [catalog] does not know are always hashed; known
 * files only when they changed since the last scan (see `FileFactsModel`). The [ScanListener] follows the
 * progress; a cancelled coroutine stops the scan between two files.
 */
interface LibraryScanner {

    suspend fun scan(
        folder: LibraryFolderModel,
        catalog: CatalogModel,
        listener: ScanListener = ScanListener.NONE,
    ): StorageResult<FolderSnapshotModel>
}
