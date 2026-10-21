package dms.application.library

import dms.application.library.model.ScanProgress
import dms.application.library.model.ScanResult
import dms.application.library.model.ScannedFile
import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.ScanListener
import dms.application.port.storage.model.StorageResult
import dms.domain.change.ChangeDetector
import dms.domain.change.FileFacts
import dms.domain.change.FolderSnapshot
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Scans every library folder and reports how they differ from the catalog.
 *
 * With `addNewFiles`, files of the primary folder that the catalog does not know are catalogued while
 * the scan reads them (see [AddScannedFiles]), so they show up in the library long before the scan
 * ends. Otherwise the scan changes nothing.
 */
class ScanLibrary(
    private val store: CatalogStore,
    private val scanner: LibraryScanner,
    private val addScannedFiles: AddScannedFiles,
) {

    /**
     * Returns `null` while no library is open. [onProgress] follows the scan file by file; it is
     * called on the scanning thread.
     */
    suspend operator fun invoke(addNewFiles: Boolean = false, onProgress: (ScanProgress) -> Unit = {}): ScanResult? {
        val folders = store.current()?.folders?.folders ?: return null
        val snapshots = mutableListOf<FolderSnapshot>()
        val failures = mutableMapOf<LibraryFolderId, StorageResult.Failure>()
        folders.forEachIndexed { index, folder ->
            val start = ScanProgress(
                folderName = folder.displayName,
                folderNumber = index + 1,
                folderCount = folders.size,
            )
            onProgress(start)
            val adding = addNewFiles && folder.role == FolderRole.PRIMARY
            when (val result = scanFolder(folder = folder, start = start, adding = adding, onProgress = onProgress)) {
                is StorageResult.Success -> snapshots += result.value
                is StorageResult.Failure -> failures[folder.id] = result
                null -> return null
            }
        }
        val catalog = store.current()?.catalog ?: return null
        return ScanResult(
            changes = ChangeDetector.detect(catalog = catalog, snapshots = snapshots),
            scannedFolders = snapshots.map { it.folderId }.toSet(),
            failedFolders = failures,
        )
    }

    /** Scans one folder against the newest catalog; while [adding], catalogs new files every few seconds. */
    private suspend fun scanFolder(
        folder: LibraryFolder,
        start: ScanProgress,
        adding: Boolean,
        onProgress: (ScanProgress) -> Unit,
    ): StorageResult<FolderSnapshot>? {
        val catalog = store.current()?.catalog ?: return null
        val knownPaths = catalog.documents.values.map { it.path }.toSet()
        val found = ConcurrentLinkedQueue<ScannedFile>()
        val listener = object : ScanListener {
            private var progress = start

            override fun counted(total: Int) {
                progress = progress.copy(total = total)
                onProgress(progress)
            }

            override fun fileDone(done: Int, path: RelativePath, facts: FileFacts, hashed: Boolean) {
                val contentHash = facts.contentHash
                if (adding && contentHash != null && path !in knownPaths) {
                    found += ScannedFile(path = path, size = facts.size, contentHash = contentHash)
                }
                progress = progress.copy(done = done, current = path, hashed = hashed)
                onProgress(progress)
            }
        }
        return coroutineScope {
            val scanning = AtomicBoolean(true)
            val adder = if (adding) {
                launch {
                    while (scanning.get()) {
                        delay(timeMillis = ADD_INTERVAL_MILLIS)
                        addFound(found = found)
                    }
                }
            } else {
                null
            }
            try {
                scanner.scan(folder = folder, catalog = catalog, listener = listener)
            } finally {
                scanning.set(false)
                adder?.cancel()
                addFound(found = found)
            }
        }
    }

    /** Catalogs what was found so far. Not cancellable, so a catalog write is never cut in half. */
    private suspend fun addFound(found: ConcurrentLinkedQueue<ScannedFile>) {
        withContext(context = NonCancellable) {
            val batch = generateSequence { found.poll() }.toList()
            addScannedFiles(files = batch)
        }
    }

    private companion object {
        const val ADD_INTERVAL_MILLIS = 2_000L
    }
}
