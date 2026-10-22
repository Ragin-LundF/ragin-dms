package dms.ui.support

import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import dms.application.library.model.LibraryCommand
import dms.application.port.state.model.PreferencesModel
import dms.domain.library.model.FolderLocationModel
import dms.storage.platform.SystemTrash
import dms.ui.AppContainer
import dms.ui.library.LibraryViewModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import java.nio.file.Path
import java.util.concurrent.atomic.AtomicInteger
import kotlin.io.path.createTempDirectory

/**
 * The real app on temporary folders: a primary, a sync folder, a trash and the app data directory,
 * with a fake AI server. Only synthetic file names and contents.
 */
class TestApp(scope: CoroutineScope) {

    val root: Path = createTempDirectory(prefix = "dms-ui-")
    val primary: Path = Files.createDirectories(root.resolve("primary"))
    val sync: Path = Files.createDirectories(root.resolve("sync"))
    val trash: Path = Files.createDirectories(root.resolve("trash"))
    val platform: FakePlatform = FakePlatform()
    val ai: FakeAiServer = FakeAiServer()

    private val trashed = AtomicInteger()
    private val viewModels = ViewModelStore()
    private val viewModelKeys = AtomicInteger()

    val services: AppServices = AppContainer(
        appDataDirectory = root.resolve("app-data"),
        trash = SystemTrash { _, entry ->
            Files.move(entry, trash.resolve("${trashed.incrementAndGet()}-${entry.fileName}"))
            true
        },
        aiEngine = ai.engine,
    ).services

    val session: LibrarySession = LibrarySession(services = services, scope = scope)

    /**
     * Configures the primary and, with [withSync], the sync folder, and opens the library. New files
     * are offered on the dashboard unless [addNewFiles] is set.
     */
    suspend fun configure(withSync: Boolean = true, addNewFiles: Boolean = false) {
        services.library.settings.savePreferences(
            preferences = PreferencesModel(addNewFilesWhileScanning = addNewFiles),
        )
        services.library.configureLibrary(
            command = LibraryCommand.CreatePrimary(
                displayName = "Primary",
                location = FolderLocationModel(value = "$primary"),
            ),
        )
        if (withSync) {
            services.library.configureLibrary(
                command = LibraryCommand.AddSyncFolder(
                    displayName = "Sync",
                    location = FolderLocationModel(value = "$sync"),
                ),
            )
        }
        session.open()
        idle()
    }

    /** Waits until the running operation finished. */
    suspend fun idle() {
        session.state.first { it.operations.isEmpty() }
    }

    /**
     * A new library view model, cleared in [close] as the app clears it when its screen leaves, so
     * its work in the background does not run on into the next test.
     */
    fun libraryViewModel(): LibraryViewModel {
        val factory = viewModelFactory {
            initializer { LibraryViewModel(services = services, session = session, platform = platform) }
        }
        val provider = ViewModelProvider.create(store = viewModels, factory = factory)
        return provider["library-${viewModelKeys.incrementAndGet()}", LibraryViewModel::class]
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
        viewModels.clear()
        root.toFile().deleteRecursively()
    }
}
