package dms.storage.state.dto

/** `settings.yaml`: the library folders and the preferences of this device. */
data class SettingsDto(
    val folders: List<LibraryFolderDto> = emptyList(),
    val addNewFilesWhileScanning: Boolean = true,
)
