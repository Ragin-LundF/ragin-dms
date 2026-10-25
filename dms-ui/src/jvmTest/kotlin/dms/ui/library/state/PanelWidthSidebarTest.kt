package dms.ui.library.state

import androidx.compose.ui.unit.dp
import dms.application.port.state.model.LayoutPreferencesModel
import dms.application.port.state.model.PreferencesModel
import dms.ui.library.state.model.PanelLayoutModel
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** [PanelWidth] for the library sidebar. */
class PanelWidthSidebarTest {

    @Test
    fun `a drag shows the new width within its limits and saves it only when it ends`() {
        UiTest.run { app ->
            val sidebar = app.libraryViewModel().sidebarWidth

            sidebar.resize(delta = 40.dp)
            sidebar.resize(delta = 400.dp)
            val dragging = sidebar.state.value
            val storedWhileDragging = app.session.state.value.preferences.layout.sidebarWidth
            sidebar.endResize()

            assertEquals(expected = PanelLayoutModel(width = 360, folded = false), actual = dragging)
            assertEquals(expected = LayoutPreferencesModel.DEFAULT_SIDEBAR_WIDTH, actual = storedWhileDragging)
            assertEquals(
                expected = 360,
                actual = savedOnce(app = app) { it.layout.sidebarWidth == 360 }.layout.sidebarWidth,
            )
            assertEquals(expected = 360, actual = sidebar.state.first { it.width == 360 }.width)
        }
    }

    @Test
    fun `dragging well below the narrowest width folds the sidebar and keeps its width`() {
        UiTest.run { app ->
            val sidebar = app.libraryViewModel().sidebarWidth

            sidebar.resize(delta = (-60).dp)
            sidebar.endResize()
            val stopsAtMinimum = savedOnce(app = app) { it.layout.sidebarWidth == 150 }.layout
            sidebar.resize(delta = (-51).dp)
            sidebar.endResize()

            val folded = savedOnce(app = app) { it.layout.sidebarFolded }.layout
            assertFalse(actual = stopsAtMinimum.sidebarFolded)
            assertTrue(actual = folded.sidebarFolded)
            assertEquals(expected = 150, actual = folded.sidebarWidth)
            assertEquals(
                expected = PanelLayoutModel(width = 150, folded = true),
                actual = sidebar.state.first { it.folded },
            )
        }
    }

    @Test
    fun `the panel button folds and unfolds, and a double-click resets the width`() {
        UiTest.run { app ->
            val library = app.libraryViewModel()
            library.sidebarWidth.resize(delta = 80.dp)
            library.sidebarWidth.endResize()
            savedOnce(app = app) { it.layout.sidebarWidth == 280 }

            library.sidebarWidth.toggleFolded()
            savedOnce(app = app) { it.layout.sidebarFolded }
            library.sidebarWidth.toggleFolded()
            library.sidebarWidth.resetWidth()

            val saved = savedOnce(app = app) { !it.layout.sidebarFolded && it.layout.sidebarWidth == 200 }.layout
            assertEquals(expected = LayoutPreferencesModel(sidebarWidth = 200, sidebarFolded = false), actual = saved)
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
