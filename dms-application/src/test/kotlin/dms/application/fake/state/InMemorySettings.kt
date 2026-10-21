package dms.application.fake.state

import dms.application.port.state.SettingsRepository
import dms.domain.library.LibraryFolders

class InMemorySettings(private var folders: LibraryFolders? = null) : SettingsRepository {

    override suspend fun loadLibraryFolders(): LibraryFolders? {
        return folders
    }

    override suspend fun saveLibraryFolders(folders: LibraryFolders) {
        this.folders = folders
    }
}
