package dms.storage.state

import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import dms.application.port.state.model.LayoutPreferencesModel
import dms.application.port.state.model.PreferencesModel
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import kotlinx.coroutines.test.runTest
import tools.jackson.core.JacksonException
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.test.assertFalse
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

class FileSettingsRepositoryTest {

    private val temp: Path = Files.createTempDirectory("dms-storage-")
    private val path: Path = temp.resolve("settings.yaml")

    @AfterTest
    fun deleteTemp() {
        temp.toFile().deleteRecursively()
    }

    @Test
    fun `nothing is configured before the first save`() {
        runTest {
            assertNull(actual = FileSettingsRepository(path = path).loadLibraryFolders())
        }
    }

    @Test
    fun `the library folders survive a restart`() {
        runTest {
            val folders = LibraryFoldersModel(
                folders = listOf(
                    folder(id = "primary", location = "/data/library", role = FolderRole.PRIMARY),
                    folder(id = "sync", location = "C:\\Users\\Example\\Library", role = FolderRole.SYNC),
                ),
            )

            FileSettingsRepository(path = path).saveLibraryFolders(folders = folders)

            assertEquals(expected = folders, actual = FileSettingsRepository(path = path).loadLibraryFolders())
        }
    }

    @Test
    fun `preferences default to adding new files and are kept apart from the folders`() {
        runTest {
            val repository = FileSettingsRepository(path = path)
            val folders = LibraryFoldersModel(
                folders = listOf(folder(id = "primary", location = "/data/library", role = FolderRole.PRIMARY)),
            )
            assertEquals(expected = PreferencesModel(), actual = repository.loadPreferences())

            val preferences = PreferencesModel(
                addNewFilesWhileScanning = false,
                ai = AiSettingsModel(
                    baseUrl = "http://localhost:9999/v1",
                    model = "sample-model",
                    preferVisionForPdfs = true,
                    acceptSuggestedFileNames = true,
                    inputCharacters = 30_000,
                    provider = AiProvider.ANTHROPIC,
                    anthropicModel = "claude-sonnet-5-5",
                ),
                layout = LayoutPreferencesModel(
                    sidebarWidth = 280,
                    sidebarFolded = true,
                    folderTreeWidth = 300,
                    folderTreeHidden = true,
                ),
            )
            repository.savePreferences(preferences = preferences)
            repository.saveLibraryFolders(folders = folders)

            val reloaded = FileSettingsRepository(path = path)
            assertEquals(expected = preferences, actual = reloaded.loadPreferences())
            assertEquals(expected = folders, actual = reloaded.loadLibraryFolders())
        }
    }

    @Test
    fun `an unreadable settings file is an error, not an empty configuration`() {
        runTest {
            Files.writeString(path, "folders: [")

            assertFailsWith<JacksonException> { FileSettingsRepository(path = path).loadLibraryFolders() }
        }
    }

    private fun folder(id: String, location: String, role: FolderRole): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = id),
            displayName = "Folder $id",
            location = FolderLocationModel(value = location),
            role = role,
        )
    }

    @Test
    fun `an older AI block, or an unknown provider, loads as Local AI with the default Claude model`() {
        runTest {
            Files.writeString(path, "ai:\n  baseUrl: http://localhost:9999/v1\n  provider: SOMETHING\n")

            val ai = FileSettingsRepository(path = path).loadPreferences().ai

            assertEquals(expected = AiProvider.LOCAL, actual = ai.provider)
            assertEquals(expected = AiSettingsModel.DEFAULT_ANTHROPIC_MODEL, actual = ai.anthropicModel)
            assertEquals(expected = "http://localhost:9999/v1", actual = ai.baseUrl)
        }
    }

    @Test
    fun `AI is off on a new device, on for an older file whose AI settings were changed, and kept as saved`() {
        runTest {
            assertFalse(actual = FileSettingsRepository(path = path).loadPreferences().ai.enabled)

            Files.writeString(path, "ai:\n  model: sample-model\n")
            assertTrue(actual = FileSettingsRepository(path = path).loadPreferences().ai.enabled)

            Files.writeString(path, "ai:\n  timeoutSeconds: 300\n")
            assertFalse(actual = FileSettingsRepository(path = path).loadPreferences().ai.enabled)

            val repository = FileSettingsRepository(path = path)
            repository.savePreferences(preferences = PreferencesModel(ai = AiSettingsModel(enabled = true)))
            assertTrue(actual = FileSettingsRepository(path = path).loadPreferences().ai.enabled)
            repository.savePreferences(preferences = PreferencesModel(ai = AiSettingsModel(model = "sample-model")))
            assertFalse(actual = FileSettingsRepository(path = path).loadPreferences().ai.enabled)
        }
    }
}
