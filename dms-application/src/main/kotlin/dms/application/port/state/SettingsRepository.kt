package dms.application.port.state

import dms.application.port.state.model.Preferences
import dms.domain.library.LibraryFolders

/** Settings of this device. Folder locations differ per device and are never stored in the catalog. */
interface SettingsRepository {

    /** The configured library folders, or `null` before the first configuration. */
    suspend fun loadLibraryFolders(): LibraryFolders?

    suspend fun saveLibraryFolders(folders: LibraryFolders)

    /** The preferences of this device, with defaults for everything never set. */
    suspend fun loadPreferences(): Preferences

    suspend fun savePreferences(preferences: Preferences)
}
