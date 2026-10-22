package dms.application.fake.state

import dms.application.port.state.SettingsRepository
import dms.application.port.state.model.PreferencesModel
import dms.domain.library.model.LibraryFoldersModel

class InMemorySettings(private var folders: LibraryFoldersModel? = null) : SettingsRepository {

    override suspend fun loadLibraryFolders(): LibraryFoldersModel? {
        return folders
    }

    override suspend fun saveLibraryFolders(folders: LibraryFoldersModel) {
        this.folders = folders
    }

    private var preferences = PreferencesModel()

    override suspend fun loadPreferences(): PreferencesModel {
        return preferences
    }

    override suspend fun savePreferences(preferences: PreferencesModel) {
        this.preferences = preferences
    }
}
