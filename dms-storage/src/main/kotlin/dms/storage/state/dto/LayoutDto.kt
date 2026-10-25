package dms.storage.state.dto

import dms.application.port.state.model.LayoutPreferencesModel

/** The panel layout part of `settings.yaml`. Every field is optional, so older files keep loading. */
data class LayoutDto(
    val sidebarWidth: Int = LayoutPreferencesModel.DEFAULT_SIDEBAR_WIDTH,
    val sidebarFolded: Boolean = false,
    val folderTreeWidth: Int = LayoutPreferencesModel.DEFAULT_FOLDER_TREE_WIDTH,
    val folderTreeHidden: Boolean = false,
)
