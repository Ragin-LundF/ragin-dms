package dms.ui.library.folders

import dms.domain.library.model.RelativePathModel
import dms.ui.library.folders.model.FolderCrumbModel
import dms.ui.library.folders.model.FolderTileModel
import dms.ui.support.SampleLibrary
import dms.ui.support.UiTest
import kotlin.test.Test
import kotlin.test.assertEquals

class FolderBrowserTest {

    private val home = RelativePathModel.of(raw = "Home")
    private val energy = RelativePathModel.of(raw = "Home/Energy")

    @Test
    fun `the open folder shows its path, its subfolders with counts and only the documents directly in it`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/notes.txt", "Home/Energy/power.txt", "Home/Energy/Old/gas.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope)

            browser.open(path = home)
            val state = browser.state.value

            assertEquals(
                expected = listOf(
                    FolderCrumbModel(path = RelativePathModel.ROOT, name = "Library"),
                    FolderCrumbModel(path = home, name = "Home"),
                ),
                actual = state.crumbs,
            )
            assertEquals(
                expected = listOf(FolderTileModel(path = energy, name = "Energy", documentCount = 2, folderCount = 1)),
                actual = state.tiles,
            )
            assertEquals(expected = listOf("Home/notes.txt"), actual = state.documents.map { it.path.value })
            assertEquals(expected = "Search in \"Home\"…", actual = state.searchPlaceholder)
        }
    }

    @Test
    fun `search finds documents of the open folder by name, and opening another folder clears it`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/notes.txt", "Home/invoice.txt", "Archive/invoice-old.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope)
            browser.open(path = home)

            browser.search(query = "INVOICE")
            val found = browser.state.value.documents.map { it.path.value }
            browser.open(path = RelativePathModel.of(raw = "Archive"))

            assertEquals(expected = listOf("Home/invoice.txt"), actual = found)
            assertEquals(expected = "", actual = browser.state.value.query)
        }
    }

    @Test
    fun `opening a deep folder unfolds the tree down to it, and a row folds away its subfolders`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/Old/gas.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope)

            browser.open(path = RelativePathModel.of(raw = "Home/Energy/Old"))
            val unfolded = browser.state.value.tree.map { it.path.value }
            browser.toggle(path = home)

            assertEquals(expected = listOf("Home", "Home/Energy", "Home/Energy/Old"), actual = unfolded)
            assertEquals(expected = listOf("Home"), actual = browser.state.value.tree.map { it.path.value })
        }
    }
}
