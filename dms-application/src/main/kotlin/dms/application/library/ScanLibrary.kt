package dms.application.library

import dms.application.library.model.ScanResult
import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.model.StorageResult
import dms.domain.change.ChangeDetector
import dms.domain.change.FolderSnapshot
import dms.domain.library.LibraryFolderId

/** Scans every library folder and reports how they differ from the catalog. Changes nothing. */
class ScanLibrary(
    private val store: CatalogStore,
    private val scanner: LibraryScanner,
) {

    /** Returns `null` while no library is open. */
    suspend operator fun invoke(): ScanResult? {
        val library = store.current() ?: return null
        val snapshots = mutableListOf<FolderSnapshot>()
        val failures = mutableMapOf<LibraryFolderId, StorageResult.Failure>()
        library.folders.folders.forEach { folder ->
            when (val result = scanner.scan(folder = folder, catalog = library.catalog)) {
                is StorageResult.Success -> snapshots += result.value
                is StorageResult.Failure -> failures[folder.id] = result
            }
        }
        return ScanResult(
            changes = ChangeDetector.detect(catalog = library.catalog, snapshots = snapshots),
            scannedFolders = snapshots.map { it.folderId }.toSet(),
            failedFolders = failures,
        )
    }
}
