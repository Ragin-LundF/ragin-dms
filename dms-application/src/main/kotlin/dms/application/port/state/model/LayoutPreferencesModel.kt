package dms.application.port.state.model

/**
 * How the library's panels sit on this device: the width of the library sidebar and of the folder tree
 * in density-independent pixels, and whether each is folded away.
 */
data class LayoutPreferencesModel(
    val sidebarWidth: Int = DEFAULT_SIDEBAR_WIDTH,
    val sidebarFolded: Boolean = false,
    val folderTreeWidth: Int = DEFAULT_FOLDER_TREE_WIDTH,
    val folderTreeHidden: Boolean = false,
) {
    companion object {
        const val DEFAULT_SIDEBAR_WIDTH: Int = 200
        const val DEFAULT_FOLDER_TREE_WIDTH: Int = 240
        val SIDEBAR_WIDTHS: IntRange = 150..360
        val FOLDER_TREE_WIDTHS: IntRange = 160..480
    }
}
