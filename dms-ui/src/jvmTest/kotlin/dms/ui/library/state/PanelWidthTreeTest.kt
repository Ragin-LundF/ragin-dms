package dms.ui.library.state

import androidx.compose.ui.unit.dp
import dms.application.port.state.model.LayoutPreferencesModel
import dms.application.port.state.model.PreferencesModel
import dms.ui.library.folders.FolderBrowsers
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** [PanelWidth] for the folder tree. */
class PanelWidthTreeTest {

    @Test
    fun `a drag shows the new width within the limits and saves it only when it ends`() {
        UiTest.run { app ->
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope)
            val tree = browser.treeLayout

            tree.resize(delta = 150.dp)
            tree.resize(delta = 150.dp)
            val dragging = browser.state.value.treeWidth
            val storedWhileDragging = app.session.state.value.preferences.layout.folderTreeWidth
            tree.endResize()

            assertEquals(expected = LayoutPreferencesModel.FOLDER_TREE_WIDTHS.last, actual = dragging)
            assertEquals(expected = LayoutPreferencesModel.DEFAULT_FOLDER_TREE_WIDTH, actual = storedWhileDragging)
            assertEquals(expected = 480, actual = browser.state.value.treeWidth)
            val saved = savedOnce(app = app) { it.layout.folderTreeWidth == 480 }
            assertEquals(expected = 480, actual = saved.layout.folderTreeWidth)
        }
    }

    @Test
    fun `dragging well below the narrowest width hides the tree but keeps its width, and the button shows it`() {
        UiTest.run { app ->
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope)
            val tree = browser.treeLayout

            tree.resize(delta = -60.dp)
            tree.endResize()
            val narrowed = browser.state.value
            savedOnce(app = app) { it.layout.folderTreeWidth == 180 }
            tree.resize(delta = -150.dp)
            tree.endResize()
            val hidden = browser.state.value
            val saved = savedOnce(app = app) { it.layout.folderTreeHidden }
            tree.toggleFolded()

            assertEquals(expected = 180, actual = narrowed.treeWidth)
            assertFalse(actual = narrowed.treeHidden)
            assertTrue(actual = hidden.treeHidden)
            assertEquals(expected = 180, actual = hidden.treeWidth)
            assertEquals(expected = 180, actual = saved.layout.folderTreeWidth)
            assertFalse(actual = browser.state.value.treeHidden)
        }
    }

    @Test
    fun `resetting the width goes back to the default`() {
        UiTest.run { app ->
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope)
            browser.treeLayout.resize(delta = 100.dp)
            browser.treeLayout.endResize()

            browser.treeLayout.resetWidth()

            assertEquals(
                expected = LayoutPreferencesModel.DEFAULT_FOLDER_TREE_WIDTH,
                actual = browser.state.value.treeWidth,
            )
        }
    }

    /**
     * The saved preferences once [done] holds for them and for the session's; the file is written on the
     * IO dispatcher, in real time, and the session takes the saved preferences right after.
     */
    private suspend fun savedOnce(app: TestApp, done: (PreferencesModel) -> Boolean): PreferencesModel {
        return withContext(context = Dispatchers.Default) {
            withTimeout(timeMillis = 5_000) {
                var saved = app.services.library.settings.loadPreferences()
                while (!done(saved) || !done(app.session.state.value.preferences)) {
                    delay(timeMillis = 10)
                    saved = app.services.library.settings.loadPreferences()
                }
                saved
            }
        }
    }
}
