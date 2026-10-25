package dms.application.folder

import dms.application.TestLibrary
import dms.application.folder.model.FolderEntriesOutcome
import dms.application.port.storage.model.FolderEntriesModel
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class ReadFolderEntriesTest {

    private fun path(raw: String): RelativePathModel {
        return RelativePathModel.of(raw = raw)
    }

    @Test
    fun `empty folders, uncatalogued files and files only in a sync folder are all found`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "home/energy/power.pdf", content = "power")
            library.createFolder(path = path(raw = "home/empty"))
            library.folders.put(folder = library.primary, path = "home/energy/new-scan.pdf", content = "scan")
            library.folders.put(folder = library.syncA, path = "home/energy/only-here.pdf", content = "sync")

            val outcome = library.readFolderEntries(path = path(raw = "home"))
            val read = assertIs<FolderEntriesOutcome.Read>(value = outcome)

            assertEquals(
                expected = FolderEntriesModel(
                    folders = setOf(path(raw = "home/energy"), path(raw = "home/empty")),
                    files = setOf(
                        path(raw = "home/energy/power.pdf"),
                        path(raw = "home/energy/new-scan.pdf"),
                        path(raw = "home/energy/only-here.pdf"),
                    ),
                ),
                actual = read.entries,
            )
        }
    }

    @Test
    fun `an offline sync folder adds nothing, and a missing folder is empty`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "home/power.pdf", content = "power")
            library.folders.unreachable += library.syncB.id

            val home = assertIs<FolderEntriesOutcome.Read>(value = library.readFolderEntries(path = path(raw = "home")))
            val missing = assertIs<FolderEntriesOutcome.Read>(value = library.readFolderEntries(path = path(raw = "x")))

            assertEquals(expected = setOf(path(raw = "home/power.pdf")), actual = home.entries.files)
            assertEquals(expected = FolderEntriesModel(), actual = missing.entries)
        }
    }

    @Test
    fun `nothing is read while no library is open`() {
        runTest {
            assertEquals(expected = FolderEntriesOutcome.LibraryNotOpen, actual = TestLibrary().readFolderEntries())
        }
    }
}
