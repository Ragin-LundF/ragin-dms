package dms.ui.library.state

import androidx.compose.ui.unit.Dp
import dms.application.port.state.model.LayoutPreferencesModel
import dms.ui.library.state.model.LayoutPanel
import dms.ui.library.state.model.PanelLayoutModel
import dms.ui.session.LayoutPreferences
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlin.math.roundToInt

/**
 * The width of a library [panel] and whether it is folded away, for the sidebar and the folder tree
 * alike. A drag only changes the width shown; its end saves it through [layout], the one writer of the
 * panel layout. Dragging well below the narrowest width folds the panel and keeps its width, and a
 * double-click on its edge resets the width.
 */
class PanelWidth(
    private val panel: LayoutPanel,
    private val layout: LayoutPreferences,
    stored: Flow<LayoutPreferencesModel>,
    private val current: () -> LayoutPreferencesModel,
    scope: CoroutineScope,
) {

    private val dragged = MutableStateFlow<Float?>(value = null)

    val state: StateFlow<PanelLayoutModel> = combine(flow = stored, flow2 = dragged) { preferences, drag ->
        PanelLayoutModel(
            width = drag?.let { clamped(width = it) } ?: widthOf(preferences = preferences),
            folded = foldedOf(preferences = preferences),
        )
    }.stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = layoutOf(preferences = current()))

    /** Widens the panel by [delta], or narrows it when negative; saved only when the drag ends. */
    fun resize(delta: Dp) {
        dragged.value = (dragged.value ?: widthOf(preferences = current()).toFloat()) + delta.value
    }

    /** Saves the dragged width, or folds the panel and keeps its width when dragged far below the narrowest. */
    fun endResize() {
        val width = dragged.value ?: return
        dragged.value = null
        if (width < widths().first - FOLD_DISTANCE) {
            layout.update { withFolded(preferences = it, folded = true) }
            return
        }
        layout.update { withWidth(preferences = it, width = clamped(width = width)) }
    }

    fun resetWidth() {
        layout.update { withWidth(preferences = it, width = defaultWidth()) }
    }

    fun toggleFolded() {
        layout.update { withFolded(preferences = it, folded = !foldedOf(preferences = it)) }
    }

    private fun layoutOf(preferences: LayoutPreferencesModel): PanelLayoutModel {
        return PanelLayoutModel(
            width = widthOf(preferences = preferences),
            folded = foldedOf(preferences = preferences),
        )
    }

    private fun clamped(width: Float): Int {
        return width.roundToInt().coerceIn(range = widths())
    }

    private fun widths(): IntRange {
        return when (panel) {
            LayoutPanel.SIDEBAR -> LayoutPreferencesModel.SIDEBAR_WIDTHS
            LayoutPanel.FOLDER_TREE -> LayoutPreferencesModel.FOLDER_TREE_WIDTHS
        }
    }

    private fun defaultWidth(): Int {
        return when (panel) {
            LayoutPanel.SIDEBAR -> LayoutPreferencesModel.DEFAULT_SIDEBAR_WIDTH
            LayoutPanel.FOLDER_TREE -> LayoutPreferencesModel.DEFAULT_FOLDER_TREE_WIDTH
        }
    }

    private fun widthOf(preferences: LayoutPreferencesModel): Int {
        return when (panel) {
            LayoutPanel.SIDEBAR -> preferences.sidebarWidth
            LayoutPanel.FOLDER_TREE -> preferences.folderTreeWidth
        }
    }

    private fun foldedOf(preferences: LayoutPreferencesModel): Boolean {
        return when (panel) {
            LayoutPanel.SIDEBAR -> preferences.sidebarFolded
            LayoutPanel.FOLDER_TREE -> preferences.folderTreeHidden
        }
    }

    private fun withWidth(preferences: LayoutPreferencesModel, width: Int): LayoutPreferencesModel {
        return when (panel) {
            LayoutPanel.SIDEBAR -> preferences.copy(sidebarWidth = width)
            LayoutPanel.FOLDER_TREE -> preferences.copy(folderTreeWidth = width)
        }
    }

    private fun withFolded(preferences: LayoutPreferencesModel, folded: Boolean): LayoutPreferencesModel {
        return when (panel) {
            LayoutPanel.SIDEBAR -> preferences.copy(sidebarFolded = folded)
            LayoutPanel.FOLDER_TREE -> preferences.copy(folderTreeHidden = folded)
        }
    }

    private companion object {
        /** How far below the narrowest width a drag must go to fold the panel, in dp. */
        const val FOLD_DISTANCE = 50
    }
}
