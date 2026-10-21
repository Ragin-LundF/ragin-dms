package dms.storage.state

import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders
import kotlinx.coroutines.test.runTest
import tools.jackson.core.JacksonException
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
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
            val folders = LibraryFolders(
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
    fun `an unreadable settings file is an error, not an empty configuration`() {
        runTest {
            Files.writeString(path, "folders: [")

            assertFailsWith<JacksonException> { FileSettingsRepository(path = path).loadLibraryFolders() }
        }
    }

    private fun folder(id: String, location: String, role: FolderRole): LibraryFolder {
        return LibraryFolder(
            id = LibraryFolderId(value = id),
            displayName = "Folder $id",
            location = FolderLocation(value = location),
            role = role,
        )
    }
}
