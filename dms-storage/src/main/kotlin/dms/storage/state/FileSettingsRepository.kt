package dms.storage.state

import dms.application.port.state.SettingsRepository
import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import dms.application.port.state.model.LayoutPreferencesModel
import dms.application.port.state.model.PreferencesModel
import dms.domain.library.FolderRole
import dms.domain.library.model.FolderLocationModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.LibraryFoldersModel
import dms.storage.state.dto.AiSettingsDto
import dms.storage.state.dto.LayoutDto
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

    override suspend fun loadLibraryFolders(): LibraryFoldersModel? {
        return withContext(context = Dispatchers.IO) {
            val folders = file.read()?.folders.orEmpty()
            if (folders.isEmpty()) null else LibraryFoldersModel(folders = folders.map { toDomain(dto = it) })
        }
    }

    override suspend fun saveLibraryFolders(folders: LibraryFoldersModel) {
        withContext(context = Dispatchers.IO) {
            val current = file.read() ?: SettingsDto()
            file.write(value = current.copy(folders = folders.folders.map { toDto(folder = it) }))
        }
    }

    override suspend fun loadPreferences(): PreferencesModel {
        return withContext(context = Dispatchers.IO) {
            val settings = file.read() ?: SettingsDto()
            PreferencesModel(
                addNewFilesWhileScanning = settings.addNewFilesWhileScanning,
                ai = AiSettingsModel(
                    baseUrl = settings.ai.baseUrl,
                    model = settings.ai.model,
                    preferVisionForPdfs = settings.ai.preferVisionForPdfs,
                    timeoutSeconds = settings.ai.timeoutSeconds,
                    acceptSuggestedFileNames = settings.ai.acceptSuggestedFileNames,
                    embeddingModel = settings.ai.embeddingModel,
                    // A hand-edited value out of range is moved into it instead of making the file unreadable.
                    inputCharacters = settings.ai.inputCharacters
                        .coerceIn(range = AiSettingsModel.INPUT_CHARACTERS_RANGE),
                    // An unknown provider falls back to the local server instead of making the file unreadable.
                    provider = AiProvider.entries.firstOrNull { it.name == settings.ai.provider } ?: AiProvider.LOCAL,
                    anthropicModel = settings.ai.anthropicModel,
                ),
                layout = LayoutPreferencesModel(
                    sidebarWidth = settings.layout.sidebarWidth.coerceIn(range = LayoutPreferencesModel.SIDEBAR_WIDTHS),
                    sidebarFolded = settings.layout.sidebarFolded,
                    folderTreeWidth = settings.layout.folderTreeWidth
                        .coerceIn(range = LayoutPreferencesModel.FOLDER_TREE_WIDTHS),
                    folderTreeHidden = settings.layout.folderTreeHidden,
                ),
            )
        }
    }

    override suspend fun savePreferences(preferences: PreferencesModel) {
        withContext(context = Dispatchers.IO) {
            val current = file.read() ?: SettingsDto()
            val ai = preferences.ai
            val updated = current.copy(
                addNewFilesWhileScanning = preferences.addNewFilesWhileScanning,
                ai = AiSettingsDto(
                    baseUrl = ai.baseUrl,
                    model = ai.model,
                    preferVisionForPdfs = ai.preferVisionForPdfs,
                    timeoutSeconds = ai.timeoutSeconds,
                    acceptSuggestedFileNames = ai.acceptSuggestedFileNames,
                    embeddingModel = ai.embeddingModel,
                    inputCharacters = ai.inputCharacters,
                    provider = ai.provider.name,
                    anthropicModel = ai.anthropicModel,
                ),
                layout = LayoutDto(
                    sidebarWidth = preferences.layout.sidebarWidth,
                    sidebarFolded = preferences.layout.sidebarFolded,
                    folderTreeWidth = preferences.layout.folderTreeWidth,
                    folderTreeHidden = preferences.layout.folderTreeHidden,
                ),
            )
            file.write(value = updated)
        }
    }

    private fun toDto(folder: LibraryFolderModel): LibraryFolderDto {
        return LibraryFolderDto(
            id = folder.id.value,
            displayName = folder.displayName,
            location = folder.location.value,
            role = folder.role.name,
        )
    }

    private fun toDomain(dto: LibraryFolderDto): LibraryFolderModel {
        return LibraryFolderModel(
            id = LibraryFolderIdModel(value = dto.id),
            displayName = dto.displayName,
            location = FolderLocationModel(value = dto.location),
            role = FolderRole.valueOf(dto.role),
        )
    }
}
