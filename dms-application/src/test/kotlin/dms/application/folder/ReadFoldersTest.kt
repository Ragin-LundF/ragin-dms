package dms.application.folder

import dms.application.TestLibrary
import dms.application.folder.model.FoldersOutcome
import dms.application.port.storage.model.StorageFailure
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class ReadFoldersTest {

    private fun path(raw: String): RelativePathModel {
        return RelativePathModel.of(raw = raw)
    }

    @Test
    fun `the folders of the primary are read, also empty ones, and a folder only in a sync folder is not`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "home/energy/power.pdf", content = "power")
            library.createFolder(path = path(raw = "home/empty"))
            library.folders.put(folder = library.syncA, path = "elsewhere/only-here.pdf", content = "sync")

            val read = assertIs<FoldersOutcome.Read>(value = library.readFolders())

            assertEquals(
                expected = setOf(path(raw = "home"), path(raw = "home/energy"), path(raw = "home/empty")),
                actual = read.folders,
            )
        }
    }

    @Test
    fun `an unreachable primary is a failure, and nothing is read while no library is open`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.primary.id

            val failed = assertIs<FoldersOutcome.StorageFailed>(value = library.readFolders())

            assertEquals(expected = StorageFailure.FOLDER_UNREACHABLE, actual = failed.failure.reason)
            assertEquals(expected = FoldersOutcome.LibraryNotOpen, actual = TestLibrary().readFolders())
        }
    }
}
