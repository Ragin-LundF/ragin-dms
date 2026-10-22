package dms.ui.library.sidebar

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.coerceIn
import dms.ui.theme.Spacing
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

/** The width of the library sidebar, which the user drags; it keeps its width while the app runs. */
class SidebarWidth {

    private val state = MutableStateFlow(value = Spacing.sidebarWidth)

    val width: StateFlow<Dp> = state.asStateFlow()

    /** Widens the sidebar by [delta], or narrows it when negative, within its limits. */
    fun resize(delta: Dp) {
        state.update { (it + delta).coerceIn(Spacing.sidebarMinWidth, Spacing.sidebarMaxWidth) }
    }
}
