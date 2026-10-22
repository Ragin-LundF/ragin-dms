package dms.application.port.state

import dms.application.port.state.model.PreferencesModel
import dms.domain.library.model.LibraryFoldersModel

/** Settings of this device. Folder locations differ per device and are never stored in the catalog. */
interface SettingsRepository {

    /** The configured library folders, or `null` before the first configuration. */
    suspend fun loadLibraryFolders(): LibraryFoldersModel?

    suspend fun saveLibraryFolders(folders: LibraryFoldersModel)

    /** The preferences of this device, with defaults for everything never set. */
    suspend fun loadPreferences(): PreferencesModel

    suspend fun savePreferences(preferences: PreferencesModel)
}
