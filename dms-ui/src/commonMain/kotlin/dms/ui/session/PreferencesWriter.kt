package dms.ui.session

import dms.application.port.state.SettingsRepository
import dms.application.port.state.model.PreferencesModel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Writes the preferences one change at a time, so two saves never overwrite each other's change. */
class PreferencesWriter(private val settings: SettingsRepository) {

    private val lock = Mutex()

    /** Applies [change] to the newest [current] preferences, saves them and hands them to [publish]. */
    suspend fun write(
        current: () -> PreferencesModel,
        change: (PreferencesModel) -> PreferencesModel,
        publish: (PreferencesModel) -> Unit,
    ) {
        lock.withLock {
            val preferences = change(current())
            settings.savePreferences(preferences = preferences)
            publish(preferences)
        }
    }
}
