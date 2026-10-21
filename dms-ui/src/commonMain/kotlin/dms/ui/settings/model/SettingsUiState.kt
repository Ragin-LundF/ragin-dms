package dms.ui.settings.model

/** The settings screen: the library folders of this device. */
data class SettingsUiState(
    val configured: Boolean = false,
    val rows: List<FolderRow> = emptyList(),
    val busy: String? = null,
)
