package dms.domain.library.model

import dms.domain.library.FolderRole

/** One folder that holds a complete copy of the library. */
data class LibraryFolderModel(
    val id: LibraryFolderIdModel,
    val displayName: String,
    val location: FolderLocationModel,
    val role: FolderRole,
) {
    init {
        require(value = displayName.isNotBlank()) { "Display name must not be blank" }
    }
}
