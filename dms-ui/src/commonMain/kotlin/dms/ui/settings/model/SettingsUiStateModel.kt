package dms.ui.settings.model

import dms.application.port.state.model.AiSettingsModel

/** The settings screen: the library folders and the preferences of this device. */
data class SettingsUiStateModel(
    val configured: Boolean = false,
    val rows: List<FolderRowModel> = emptyList(),
    val busy: String? = null,
    val addNewFilesWhileScanning: Boolean = true,
    val ai: AiSettingsModel = AiSettingsModel(),
    val aiConnection: AiConnectionModel = AiConnectionModel(),
)
