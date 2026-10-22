package dms.storage.watch

import dms.application.port.storage.FolderWatcher
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.storage.fs.PathResolver
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.io.IOException
import java.nio.file.ClosedWatchServiceException
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardWatchEventKinds
import java.nio.file.WatchService
import java.util.concurrent.TimeUnit
import kotlin.io.path.isDirectory

/**
 * Watches a folder below every library folder with the platform's `WatchService` and emits on every
 * change. Only a hint: network shares and cloud folders may not report changes at all, which is why
 * the scan, not the watcher, decides what changed. Subfolders that exist when watching starts are
 * watched too.
 */
class FileSystemFolderWatcher : FolderWatcher {

    override fun changes(folders: List<LibraryFolderModel>, path: RelativePathModel): Flow<Unit> {
        return callbackFlow {
            val service = FileSystems.getDefault().newWatchService()
            folders.mapNotNull { folder -> PathResolver.root(folder = folder) }
                .mapNotNull { root -> PathResolver.existing(root = root, path = path) }
                .filter { it.isDirectory() }
                .forEach { register(service = service, directory = it) }
            val poller = launch(context = Dispatchers.IO) {
                while (isActive) {
                    val key = runCatching { service.poll(POLL_MILLIS, TimeUnit.MILLISECONDS) }.getOrNull() ?: continue
                    if (key.pollEvents().isNotEmpty()) {
                        trySend(element = Unit)
                    }
                    key.reset()
                }
            }
            awaitClose {
                poller.cancel()
                runCatching { service.close() }
            }
        }.flowOn(context = Dispatchers.IO)
    }

    private fun register(service: WatchService, directory: Path) {
        try {
            Files.walk(directory).use { paths ->
                paths.filter { Files.isDirectory(it) }.forEach { dir ->
                    dir.register(
                        service,
                        StandardWatchEventKinds.ENTRY_CREATE,
                        StandardWatchEventKinds.ENTRY_MODIFY,
                        StandardWatchEventKinds.ENTRY_DELETE,
                    )
                }
            }
        } catch (ignored: IOException) {
            // A folder that cannot be watched is left to the next scan.
        } catch (ignored: ClosedWatchServiceException) {
            // Watching stopped while registering.
        }
    }

    private companion object {
        const val POLL_MILLIS = 500L
    }
}
