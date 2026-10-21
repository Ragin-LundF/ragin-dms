package dms.ui.settings.model

import dms.application.port.state.model.AiSettings

/** The settings screen: the library folders and the preferences of this device. */
data class SettingsUiState(
    val configured: Boolean = false,
    val rows: List<FolderRow> = emptyList(),
    val busy: String? = null,
    val addNewFilesWhileScanning: Boolean = true,
    val ai: AiSettings = AiSettings(),
    val aiConnection: AiConnection = AiConnection(),
)
