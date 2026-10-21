package dms.storage.state

import dms.application.port.state.SettingsRepository
import dms.application.port.state.model.Preferences
import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders
import dms.storage.state.dto.LibraryFolderDto
import dms.storage.state.dto.SettingsDto
import dms.storage.yaml.YamlFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.nio.file.Path

/**
 * `settings.yaml` in the app data folder. An unreadable file is an error, never "not configured":
 * otherwise a new configuration would silently replace the existing one.
 */
class FileSettingsRepository(path: Path) : SettingsRepository {

    private val file = YamlFile(path = path, type = SettingsDto::class.java)

    override suspend fun loadLibraryFolders(): LibraryFolders? {
        return withContext(context = Dispatchers.IO) {
            val folders = file.read()?.folders.orEmpty()
            if (folders.isEmpty()) null else LibraryFolders(folders = folders.map { toDomain(dto = it) })
        }
    }

    override suspend fun saveLibraryFolders(folders: LibraryFolders) {
        withContext(context = Dispatchers.IO) {
            val current = file.read() ?: SettingsDto()
            file.write(value = current.copy(folders = folders.folders.map { toDto(folder = it) }))
        }
    }

    override suspend fun loadPreferences(): Preferences {
        return withContext(context = Dispatchers.IO) {
            val settings = file.read() ?: SettingsDto()
            Preferences(addNewFilesWhileScanning = settings.addNewFilesWhileScanning)
        }
    }

    override suspend fun savePreferences(preferences: Preferences) {
        withContext(context = Dispatchers.IO) {
            val current = file.read() ?: SettingsDto()
            file.write(value = current.copy(addNewFilesWhileScanning = preferences.addNewFilesWhileScanning))
        }
    }

    private fun toDto(folder: LibraryFolder): LibraryFolderDto {
        return LibraryFolderDto(
            id = folder.id.value,
            displayName = folder.displayName,
            location = folder.location.value,
            role = folder.role.name,
        )
    }

    private fun toDomain(dto: LibraryFolderDto): LibraryFolder {
        return LibraryFolder(
            id = LibraryFolderId(value = dto.id),
            displayName = dto.displayName,
            location = FolderLocation(value = dto.location),
            role = FolderRole.valueOf(dto.role),
        )
    }
}
