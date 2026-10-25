package dms.application.library

import dms.application.library.model.LibraryCommand
import dms.application.library.model.outcome.ConfigureLibraryOutcome
import dms.application.port.state.SettingsRepository
import dms.application.port.system.IdGenerator
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel

/** Sets up the primary folder and adds, removes or promotes sync folders on this device. */
class ConfigureLibrary(
    private val settings: SettingsRepository,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(command: LibraryCommand): ConfigureLibraryOutcome {
        val current = settings.loadLibraryFolders()
        if (command is LibraryCommand.CreatePrimary) {
            return createPrimary(current = current, command = command)
        }
        if (current == null) {
            return ConfigureLibraryOutcome.NotConfigured
        }
        val outcome = when (command) {
            is LibraryCommand.AddSyncFolder -> addSyncFolder(current = current, command = command)
            is LibraryCommand.RemoveSyncFolder -> removeSyncFolder(current = current, folderId = command.folderId)
            is LibraryCommand.ChoosePrimary -> choosePrimary(current = current, folderId = command.folderId)
            is LibraryCommand.CreatePrimary -> ConfigureLibraryOutcome.AlreadyConfigured
        }
        if (outcome is ConfigureLibraryOutcome.Configured) {
            settings.saveLibraryFolders(folders = outcome.folders)
        }
        return outcome
    }

    private suspend fun createPrimary(
        current: LibraryFoldersModel?,
        command: LibraryCommand.CreatePrimary,
    ): ConfigureLibraryOutcome {
        if (current != null) {
            return ConfigureLibraryOutcome.AlreadyConfigured
        }
        val primary = newFolder(
            displayName = command.displayName,
            location = command.location,
            role = FolderRole.PRIMARY,
        )
        val folders = LibraryFoldersModel(folders = listOf(primary))
        settings.saveLibraryFolders(folders = folders)
        return ConfigureLibraryOutcome.Configured(folders = folders)
    }

    private fun addSyncFolder(
        current: LibraryFoldersModel,
        command: LibraryCommand.AddSyncFolder,
    ): ConfigureLibraryOutcome {
        if (current.contains(location = command.location)) {
            return ConfigureLibraryOutcome.LocationAlreadyUsed
        }
        val folder = newFolder(displayName = command.displayName, location = command.location, role = FolderRole.SYNC)
        return ConfigureLibraryOutcome.Configured(folders = current.add(folder = folder))
    }

    private fun removeSyncFolder(
        current: LibraryFoldersModel,
        folderId: LibraryFolderIdModel,
    ): ConfigureLibraryOutcome {
        if (current.find(id = folderId) == null) {
            return ConfigureLibraryOutcome.UnknownFolder
        }
        if (current.primary.id == folderId) {
            return ConfigureLibraryOutcome.PrimaryCannotBeRemoved
        }
        return ConfigureLibraryOutcome.Configured(folders = current.remove(id = folderId))
    }

    private fun choosePrimary(current: LibraryFoldersModel, folderId: LibraryFolderIdModel): ConfigureLibraryOutcome {
        if (current.find(id = folderId) == null) {
            return ConfigureLibraryOutcome.UnknownFolder
        }
        return ConfigureLibraryOutcome.Configured(folders = current.withPrimary(id = folderId))
    }

    private fun newFolder(
        displayName: String,
        location: FolderLocationModel,
        role: FolderRole,
    ): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = ids.newId()),
            displayName = displayName,
            location = location,
            role = role,
        )
    }
}
