package dms.ui.library

import dms.domain.library.model.RelativePathModel
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.drag.model.FolderDropPlace
import dms.ui.support.SampleLibrary
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** How the library's parts work together: one quick look, folders dragged onto folders, the menu entries. */
class LibraryWiringTest {

    private val home = RelativePathModel.of(raw = "Home")
    private val energy = RelativePathModel.of(raw = "Home/Energy")
    private val archive = RelativePathModel.of(raw = "Archive")

    @Test
    fun `a folder dropped onto another folder moves there in every library folder`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt", "Archive/old.txt")
            val library = app.libraryViewModel()

            library.drop(
                item = DragItem.DraggedFolder(path = energy),
                target = DropTarget.Folder(path = archive, place = FolderDropPlace.TREE),
            )
            app.idle()

            assertTrue(actual = Files.exists(app.primary.resolve("Archive/Energy/power.txt")))
            assertTrue(actual = Files.exists(app.sync.resolve("Archive/Energy/power.txt")))
            val paths = SampleLibrary.catalog(app = app).documents.values.map { it.path.value }.toSet()
            assertEquals(expected = setOf("Archive/Energy/power.txt", "Archive/old.txt"), actual = paths)
        }
    }

    @Test
    fun `a folder dropped into its own subfolder is refused with a message and stays`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "Home/Energy/power.txt")
            val library = app.libraryViewModel()

            library.drop(
                item = DragItem.DraggedFolder(path = home),
                target = DropTarget.Folder(path = energy, place = FolderDropPlace.TILE),
            )
            app.idle()

            assertTrue(actual = app.session.state.value.message?.isError == true)
            assertTrue(actual = Files.exists(app.primary.resolve("Home/Energy/power.txt")))
        }
    }

    @Test
    fun `one quick look serves the library, inspecting, the menu entry and Esc alike`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "a.txt", "b.txt")
            val library = app.libraryViewModel()
            val commands = LibraryCommands()
            commands.connect(viewModel = library)
            val first = documents.getValue(key = "a.txt").id

            library.inspect(documentId = first)
            assertEquals(expected = first, actual = library.quickLook.state.value.shown)
            assertEquals(expected = first, actual = library.state.value.inspected?.id)

            commands.toggleQuickLook()
            assertNull(actual = library.quickLook.state.value.shown)
            assertNull(actual = library.state.value.inspected)

            commands.toggleQuickLook()
            assertTrue(actual = library.quickLook.isOpen)
            assertTrue(actual = library.quickLook.close())
            assertFalse(actual = library.quickLook.close())
        }
    }

    @Test
    fun `an empty folder on disk shows in the folder tree and in the bulk bar's picker, as after a restart`() {
        UiTest.run { app ->
            Files.createDirectories(app.primary.resolve("Archive/Empty"))
            SampleLibrary.open(app = app, "Home/notes.txt")
            val library = app.libraryViewModel()
            val empty = RelativePathModel.of(raw = "Archive/Empty")

            val tree = library.folders.state.first { state -> state.tree.any { it.path == archive } }.tree
            val picked = library.bulk.pickerFolders(expanded = setOf(archive)).map { it.path }

            assertTrue(actual = tree.any { it.path == archive })
            assertTrue(actual = empty in picked)
        }
    }

    @Test
    fun `without a library the menu entries do nothing`() {
        val commands = LibraryCommands()

        commands.toggleSidebar()
        commands.toggleQuickLook()
        commands.deleteFolder()

        assertFalse(actual = commands.available)
    }
}
