package dms.application.fake.state

import dms.application.port.state.SettingsRepository
import dms.application.port.state.model.Preferences
import dms.domain.library.LibraryFolders

class InMemorySettings(private var folders: LibraryFolders? = null) : SettingsRepository {

    override suspend fun loadLibraryFolders(): LibraryFolders? {
        return folders
    }

    override suspend fun saveLibraryFolders(folders: LibraryFolders) {
        this.folders = folders
    }

    private var preferences = Preferences()

    override suspend fun loadPreferences(): Preferences {
        return preferences
    }

    override suspend fun savePreferences(preferences: Preferences) {
        this.preferences = preferences
    }
}
