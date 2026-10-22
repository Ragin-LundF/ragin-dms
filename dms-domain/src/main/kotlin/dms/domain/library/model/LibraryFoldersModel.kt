package dms.domain.library.model

import dms.domain.library.FolderRole

/**
 * The folders of one library on this device: exactly one primary and any number of sync folders.
 *
 * Functions throw [IllegalArgumentException] for requests that break these rules; callers check
 * user input with [contains] and [find] first.
 */
data class LibraryFoldersModel(val folders: List<LibraryFolderModel>) {

    init {
        require(value = folders.count { it.role == FolderRole.PRIMARY } == 1) {
            "A library needs exactly one primary folder"
        }
        require(value = folders.distinctBy { it.id }.size == folders.size) { "Library folder ids must be unique" }
        require(value = folders.distinctBy { it.location }.size == folders.size) {
            "A location can only be used by one library folder"
        }
    }

    val primary: LibraryFolderModel
        get() {
            return folders.first { it.role == FolderRole.PRIMARY }
        }

    val syncFolders: List<LibraryFolderModel>
        get() {
            return folders.filter { it.role == FolderRole.SYNC }
        }

    fun find(id: LibraryFolderIdModel): LibraryFolderModel? {
        return folders.firstOrNull { it.id == id }
    }

    fun contains(location: FolderLocationModel): Boolean {
        return folders.any { it.location == location }
    }

    /** Adds a sync folder. */
    fun add(folder: LibraryFolderModel): LibraryFoldersModel {
        require(value = folder.role == FolderRole.SYNC) { "Only sync folders can be added; use withPrimary" }
        return LibraryFoldersModel(folders = folders + folder)
    }

    /** Removes a sync folder. The primary cannot be removed; choose another primary first. */
    fun remove(id: LibraryFolderIdModel): LibraryFoldersModel {
        require(value = primary.id != id) { "The primary folder cannot be removed" }
        return LibraryFoldersModel(folders = folders.filterNot { it.id == id })
    }

    /** Makes [id] the primary folder; the previous primary becomes a sync folder. */
    fun withPrimary(id: LibraryFolderIdModel): LibraryFoldersModel {
        require(value = find(id = id) != null) { "Unknown library folder: $id" }
        return LibraryFoldersModel(
            folders = folders.map { folder ->
                val role = if (folder.id == id) FolderRole.PRIMARY else FolderRole.SYNC
                folder.copy(role = role)
            },
        )
    }
}
