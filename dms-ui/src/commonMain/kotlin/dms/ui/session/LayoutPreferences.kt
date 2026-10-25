package dms.ui.session

import dms.application.port.state.model.LayoutPreferencesModel
import dms.ui.session.model.SessionStateModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * How the library's panels sit (sidebar and folder tree: width and folded state), part of the session's
 * preferences. It changes nothing in the library, so saving runs beside every lane instead of waiting
 * for a scan.
 */
class LayoutPreferences(
    private val state: MutableStateFlow<SessionStateModel>,
    private val writer: PreferencesWriter,
    private val scope: CoroutineScope,
) {

    /**
     * Applies [change] at once, so the panel takes its new size, and saves it. Call it when a drag
     * ends, not on every step of it.
     */
    fun update(change: (LayoutPreferencesModel) -> LayoutPreferencesModel) {
        state.update { it.copy(preferences = it.preferences.copy(layout = change(it.preferences.layout))) }
        scope.launch {
            // Saves the newest layout, and never puts an older one back over a change made meanwhile.
            writer.write(
                current = { state.value.preferences },
                change = { saved -> saved.copy(layout = state.value.preferences.layout) },
                publish = { saved ->
                    state.update { it.copy(preferences = saved.copy(layout = it.preferences.layout)) }
                },
            )
        }
    }
}
