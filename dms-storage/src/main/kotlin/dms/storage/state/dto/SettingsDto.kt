package dms.storage.state.dto

/** `settings.yaml`: the library folders of this device. */
data class SettingsDto(
    val folders: List<LibraryFolderDto> = emptyList(),
)
