package dms.domain.library

/** One folder that holds a complete copy of the library. */
data class LibraryFolder(
    val id: LibraryFolderId,
    val displayName: String,
    val location: FolderLocation,
    val role: FolderRole,
) {
    init {
        require(value = displayName.isNotBlank()) { "Display name must not be blank" }
    }
}
