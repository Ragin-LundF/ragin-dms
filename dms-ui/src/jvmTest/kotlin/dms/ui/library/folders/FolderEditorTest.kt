package dms.ui.library.folders

import dms.domain.library.model.RelativePathModel
import dms.ui.components.dialog.DialogController
import dms.ui.library.folders.model.FolderNaming
import dms.ui.session.model.UiMessageModel
import dms.ui.support.SampleLibrary
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class FolderEditorTest {

    private val dialogs = DialogController()
    private val home = RelativePathModel.of(raw = "Home")
    private val energy = RelativePathModel.of(raw = "Home/Energy")

    @Test
    fun `a new folder is named in place, saved in every library folder, and the next one is New folder 2`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/notes.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            browser.open(path = home)

            browser.newFolder()
            val naming = browser.state.value.naming
            assertEquals(expected = FolderNaming.New(parent = home, proposed = "New folder"), actual = naming)
            browser.editor.commitName(naming = checkNotNull(value = naming), text = "New folder")
            app.idle()
            // The folder is read back from disk, as after a restart.
            browser.state.first { state -> state.tiles.isNotEmpty() }
            browser.newFolder()

            assertTrue(actual = Files.isDirectory(app.primary.resolve("Home/New folder")))
            assertTrue(actual = Files.isDirectory(app.sync.resolve("Home/New folder")))
            assertEquals(expected = listOf("New folder"), actual = browser.state.value.tiles.map { it.name })
            assertEquals(
                expected = FolderNaming.New(parent = home, proposed = "New folder 2"),
                actual = browser.state.value.naming,
            )
        }
    }

    @Test
    fun `Esc creates nothing, a slash becomes a dash, and a taken name is refused with a message`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)

            browser.editor.newFolder(parent = home)
            browser.editor.cancelName()
            browser.editor.newFolder(parent = home)
            browser.editor.type(text = "Gas/Water")
            browser.editor.saveName()
            app.idle()
            browser.editor.newFolder(parent = home)
            browser.editor.type(text = "energy")
            browser.editor.saveName()
            app.idle()

            assertFalse(actual = Files.exists(app.primary.resolve("Home/New folder")))
            assertTrue(actual = Files.isDirectory(app.primary.resolve("Home/Gas-Water")))
            assertEquals(
                expected = UiMessageModel(
                    text = "Another entry there already has this name (ignoring case)",
                    isError = true,
                ),
                actual = app.session.state.value.message,
            )
            assertNull(actual = browser.state.value.naming)
        }
    }

    @Test
    fun `renaming a folder renames it in every library folder and the catalog paths inside follow`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt", "Home/Energy/Old/gas.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            browser.open(path = energy)

            browser.renameOpenFolder()
            assertEquals(expected = home, actual = browser.state.first { it.naming != null }.open)
            browser.editor.type(text = "Power")
            browser.open(path = home)
            app.idle()

            val paths = SampleLibrary.catalog(app = app).documents.values.map { it.path.value }.toSet()
            assertEquals(expected = setOf("Home/Power/power.txt", "Home/Power/Old/gas.txt"), actual = paths)
            listOf(app.primary, app.sync).forEach { folder ->
                assertTrue(actual = Files.exists(folder.resolve("Home/Power/Old/gas.txt")))
                assertFalse(actual = Files.exists(folder.resolve("Home/Energy")))
            }
            assertEquals(
                expected = UiMessageModel(text = "Renamed to \"Power\" in every library folder"),
                actual = app.session.state.value.message,
            )
        }
    }

    @Test
    fun `moving a folder into its own subfolder is refused, and the picker moves it elsewhere`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/Old/gas.txt", "Archive/old.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            val editor = browser.editor

            editor.moveInto(path = energy, folder = RelativePathModel.of(raw = "Home/Energy/Old"))
            val refused = app.session.state.value.message
            editor.chooseTarget(path = energy)
            val excluded = editor.picker.state.value?.isExcluded(path = RelativePathModel.of(raw = "Home/Energy/Old"))
            editor.picker.select(path = RelativePathModel.of(raw = "Archive"))
            editor.picker.confirm()
            app.idle()

            assertEquals(
                expected = UiMessageModel(
                    text = "A folder cannot be moved into itself or one of its subfolders",
                    isError = true,
                ),
                actual = refused,
            )
            assertEquals(expected = true, actual = excluded)
            val paths = SampleLibrary.catalog(app = app).documents.values.map { it.path.value }.toSet()
            assertEquals(expected = setOf("Archive/Energy/Old/gas.txt", "Archive/old.txt"), actual = paths)
            assertTrue(actual = Files.exists(app.sync.resolve("Archive/Energy/Old/gas.txt")))
        }
    }

    @Test
    fun `a move to where the folder already is changes nothing`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            val before = app.session.state.value

            browser.editor.moveInto(path = energy, folder = home)

            assertEquals(expected = before, actual = app.session.state.value)
        }
    }

    @Test
    fun `deleting a folder names its documents and subfolders, then removes it everywhere`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt", "Home/Energy/Old/gas.txt", "Home/notes.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            browser.open(path = energy)

            browser.deleteOpenFolder()
            val request = checkNotNull(value = dialogs.confirmation.first { it != null })
            dialogs.confirm()
            app.idle()

            assertEquals(expected = "Delete \"Energy\"?", actual = request.title)
            assertEquals(
                expected = "2 documents and 1 subfolder inside go to the trash, in every library folder.",
                actual = request.message,
            )
            val paths = SampleLibrary.catalog(app = app).documents.values.map { it.path.value }
            assertEquals(expected = listOf("Home/notes.txt"), actual = paths)
            assertFalse(actual = Files.exists(app.primary.resolve("Home/Energy")))
            assertFalse(actual = Files.exists(app.sync.resolve("Home/Energy")))
            assertEquals(expected = home, actual = browser.state.value.open)
        }
    }

    @Test
    fun `deleting an empty folder says so and nothing happens before the confirmation`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/notes.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            browser.editor.newFolder(parent = home)
            browser.editor.saveName()
            app.idle()
            val created = RelativePathModel.of(raw = "Home/New folder")
            browser.state.first { state -> state.tiles.isNotEmpty() }

            browser.editor.delete(path = created)
            val message = dialogs.confirmation.first { it != null }?.message
            dialogs.dismissConfirmation()

            assertEquals(expected = "The folder is empty. It is removed from every library folder.", actual = message)
            assertTrue(actual = Files.isDirectory(app.primary.resolve("Home/New folder")))
            assertEquals(expected = listOf(created), actual = browser.state.value.tiles.map { it.path })
        }
    }

    @Test
    fun `deleting a folder also names the files the library does not know`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt")
            Files.writeString(app.primary.resolve("Home/Energy/new-scan.txt"), "not adopted yet")
            Files.writeString(app.sync.resolve("Home/Energy/only-in-sync.txt"), "only here")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)

            browser.editor.delete(path = energy)
            val request = checkNotNull(value = dialogs.confirmation.first { it != null })
            dialogs.dismissConfirmation()

            assertEquals(
                expected = "1 document and 2 other files not in the library inside go to the trash, " +
                    "in every library folder.",
                actual = request.message,
            )
            assertEquals(
                expected = listOf(
                    "Home/Energy/power.txt",
                    "Home/Energy/new-scan.txt (not in the library)",
                    "Home/Energy/only-in-sync.txt (not in the library)",
                ),
                actual = request.affected,
            )
        }
    }

    @Test
    fun `a folder whose contents cannot be read is not offered for deletion`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)
            app.primary.toFile().deleteRecursively()

            browser.editor.delete(path = energy)
            val message = app.session.state.first { it.message != null }.message

            assertEquals(
                expected = UiMessageModel(
                    text = "\"Energy\" cannot be deleted now: its contents could not be read (folder not reachable)",
                    isError = true,
                ),
                actual = message,
            )
            assertNull(actual = dialogs.confirmation.value)
            assertTrue(actual = Files.exists(app.sync.resolve("Home/Energy/power.txt")))
        }
    }

    @Test
    fun `the top level is neither renamed nor deleted, and the user is told why`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/notes.txt")
            val browser = FolderBrowsers.of(app = app, scope = backgroundScope, dialogs = dialogs)

            browser.renameOpenFolder()
            val renamed = app.session.state.value.message?.text
            assertEquals(expected = "The library's top level cannot be renamed", actual = renamed)
            browser.deleteOpenFolder()
            val deleted = app.session.state.value.message?.text
            assertEquals(expected = "The library's top level cannot be deleted", actual = deleted)
            assertNull(actual = dialogs.confirmation.value)
        }
    }
}
