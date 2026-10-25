package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.change.DetectedChange
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class TrashDuplicateFileTest {

    @Test
    fun `a new file that is a copy of a document goes to the trash in every folder that has it`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.put(folder = library.primary, path = "inbox/a copy.pdf", content = "alpha")
            library.folders.put(folder = library.syncA, path = "inbox/a copy.pdf", content = "alpha")
            val newFile = newFileOf(library = library)

            val refused = library.trashDuplicateFile(change = newFile, confirmed = false)
            val outcome = library.trashDuplicateFile(change = newFile, confirmed = true)

            assertIs<OperationOutcome.NotConfirmed>(value = refused)
            assertIs<OperationOutcome.Done>(value = outcome)
            val copy = RelativePathModel.of(raw = "inbox/a copy.pdf")
            val expected = setOf(library.primary.id to copy, library.syncA.id to copy)
            assertEquals(expected = expected, actual = library.folders.trashed.toSet())
            val original = library.folders.content(folder = library.primary, path = "letters/a.pdf")
            assertEquals(expected = "alpha", actual = original)
        }
    }

    @Test
    fun `a file that only looks like a copy stays where it is`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.put(folder = library.syncA, path = "letters/a (1).pdf", content = "alphb")

            val outcome = library.trashDuplicateFile(change = newFileOf(library = library), confirmed = true)

            assertIs<OperationOutcome.NotFound>(value = outcome)
            assertTrue(actual = library.folders.trashed.isEmpty())
        }
    }

    private suspend fun newFileOf(library: TestLibrary): DetectedChange.NewFile {
        val scan = assertNotNull(actual = library.scanned())
        return scan.changes.filterIsInstance<DetectedChange.NewFile>().single()
    }
}
