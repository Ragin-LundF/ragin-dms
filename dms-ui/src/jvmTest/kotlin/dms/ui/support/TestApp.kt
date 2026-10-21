package dms.ui.support

import dms.application.library.model.LibraryCommand
import dms.domain.library.FolderLocation
import dms.storage.platform.SystemTrash
import dms.ui.AppContainer
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.atomic.AtomicInteger
import kotlin.io.path.createTempDirectory

/**
 * The real app on temporary folders: a primary, a sync folder, a trash and the app data directory.
 * Only synthetic file names and contents.
 */
class TestApp(scope: CoroutineScope) {

    val root: Path = createTempDirectory(prefix = "dms-ui-")
    val primary: Path = Files.createDirectories(root.resolve("primary"))
    val sync: Path = Files.createDirectories(root.resolve("sync"))
    val trash: Path = Files.createDirectories(root.resolve("trash"))
    val platform: FakePlatform = FakePlatform()

    private val trashed = AtomicInteger()

    val services: AppServices = AppContainer(
        appDataDirectory = root.resolve("app-data"),
        trash = SystemTrash { _, entry ->
            Files.move(entry, trash.resolve("${trashed.incrementAndGet()}-${entry.fileName}"))
            true
        },
    ).services

    val session: LibrarySession = LibrarySession(services = services, scope = scope)

    /** Configures the primary and, with [withSync], the sync folder, and opens the library. */
    suspend fun configure(withSync: Boolean = true) {
        services.library.configureLibrary(
            command = LibraryCommand.CreatePrimary(
                displayName = "Primary",
                location = FolderLocation(value = "$primary"),
            ),
        )
        if (withSync) {
            services.library.configureLibrary(
                command = LibraryCommand.AddSyncFolder(
                    displayName = "Sync",
                    location = FolderLocation(value = "$sync"),
                ),
            )
        }
        session.open()
        idle()
    }

    /** Waits until the running operation finished. */
    suspend fun idle() {
        session.state.first { it.busy == null }
    }

    fun write(file: Path, content: String): Path {
        Files.createDirectories(file.parent)
        Files.writeString(file, content)
        return file
    }

    fun read(file: Path): String {
        return Files.readString(file)
    }

    fun close() {
        root.toFile().deleteRecursively()
    }
}
