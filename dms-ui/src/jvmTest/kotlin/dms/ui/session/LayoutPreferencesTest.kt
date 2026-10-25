package dms.ui.session

import dms.application.port.state.model.PreferencesModel
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class LayoutPreferencesTest {

    @Test
    fun `a new panel width applies at once and is saved, even while the library is busy`() {
        UiTest.run { app ->
            val release = CompletableDeferred<Unit>()
            app.session.perform(label = "Long scan") { release.await() }

            app.session.layout.update { it.copy(sidebarWidth = 280, folderTreeHidden = true) }

            assertEquals(expected = 280, actual = app.session.state.value.preferences.layout.sidebarWidth)
            val saved = savedOnce(app = app) { it.layout.sidebarWidth == 280 }.layout
            assertEquals(expected = 280, actual = saved.sidebarWidth)
            assertTrue(actual = saved.folderTreeHidden)
            release.complete(value = Unit)
        }
    }

    @Test
    fun `a layout change keeps the other preferences`() {
        UiTest.run { app ->
            app.session.setAddNewFilesWhileScanning(enabled = false)
            app.idle()

            app.session.layout.update { it.copy(sidebarFolded = true) }

            val saved = savedOnce(app = app) { it.layout.sidebarFolded }
            assertTrue(actual = saved.layout.sidebarFolded)
            assertEquals(expected = false, actual = saved.addNewFilesWhileScanning)
        }
    }

    /** The saved preferences once [done] holds; the file is written on the IO dispatcher, in real time. */
    private suspend fun savedOnce(app: TestApp, done: (PreferencesModel) -> Boolean): PreferencesModel {
        return withContext(context = Dispatchers.Default) {
            withTimeout(timeMillis = 5_000) {
                var saved = app.services.library.settings.loadPreferences()
                while (!done(saved)) {
                    delay(timeMillis = 10)
                    saved = app.services.library.settings.loadPreferences()
                }
                saved
            }
        }
    }
}
