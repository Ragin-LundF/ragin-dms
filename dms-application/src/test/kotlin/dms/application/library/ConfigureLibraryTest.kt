package dms.application.library

import dms.application.fake.state.InMemorySettings
import dms.application.fake.system.SequentialIds
import dms.application.library.model.ConfigureLibraryOutcome
import dms.application.library.model.LibraryCommand
import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class ConfigureLibraryTest {

    private val settings = InMemorySettings()
    private val configure = ConfigureLibrary(settings = settings, ids = SequentialIds())
    private val primaryLocation = FolderLocation(value = "/library/main")
    private val backupLocation = FolderLocation(value = "/library/backup")

    @Test
    fun `the first folder becomes the primary and further folders sync folders`() {
        runTest {
            configure(command = LibraryCommand.CreatePrimary(displayName = "Main", location = primaryLocation))

            val outcome = configure(
                command = LibraryCommand.AddSyncFolder(displayName = "Backup", location = backupLocation),
            )

            val folders = assertIs<ConfigureLibraryOutcome.Configured>(value = outcome).folders
            assertEquals(expected = primaryLocation, actual = folders.primary.location)
            assertEquals(expected = listOf(backupLocation), actual = folders.syncFolders.map { it.location })
            assertEquals(expected = folders, actual = settings.loadLibraryFolders())
        }
    }

    @Test
    fun `a location can only be used once`() {
        runTest {
            configure(command = LibraryCommand.CreatePrimary(displayName = "Main", location = primaryLocation))

            val outcome = configure(
                command = LibraryCommand.AddSyncFolder(displayName = "Again", location = primaryLocation),
            )

            assertEquals(expected = ConfigureLibraryOutcome.LocationAlreadyUsed, actual = outcome)
        }
    }

    @Test
    fun `the primary cannot be removed but another folder can become the primary`() {
        runTest {
            configure(command = LibraryCommand.CreatePrimary(displayName = "Main", location = primaryLocation))
            val added = configure(
                command = LibraryCommand.AddSyncFolder(displayName = "Backup", location = backupLocation),
            )
            val folders = assertIs<ConfigureLibraryOutcome.Configured>(value = added).folders
            val backup = folders.syncFolders.single()

            val removal = configure(command = LibraryCommand.RemoveSyncFolder(folderId = folders.primary.id))
            val switched = configure(command = LibraryCommand.ChoosePrimary(folderId = backup.id))

            assertEquals(expected = ConfigureLibraryOutcome.PrimaryCannotBeRemoved, actual = removal)
            val result = assertIs<ConfigureLibraryOutcome.Configured>(value = switched).folders
            assertEquals(expected = FolderRole.PRIMARY, actual = result.find(id = backup.id)?.role)
        }
    }
}
