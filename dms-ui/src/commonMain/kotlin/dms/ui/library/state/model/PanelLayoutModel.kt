package dms.ui.library.state.model

/** How wide a panel is now in dp, also while the user drags its edge, and whether it is [folded] away. */
data class PanelLayoutModel(val width: Int, val folded: Boolean)
