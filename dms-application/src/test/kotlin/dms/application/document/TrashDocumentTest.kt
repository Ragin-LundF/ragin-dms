package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class TrashDocumentTest {

    @Test
    fun `nothing is trashed without confirmation`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")

            val outcome = library.trashDocument(documentId = document.id, confirmed = false)

            assertEquals(expected = OperationOutcome.NotConfirmed, actual = outcome)
            assertEquals(
                expected = "alpha",
                actual = library.folders.content(folder = library.primary, path = "letters/a.pdf"),
            )
        }
    }

    @Test
    fun `a confirmed document goes to the trash in every folder and leaves the catalog`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            // Already gone from one sync folder: that folder has nothing left to do.
            library.folders.moveToTrash(folder = library.syncA, path = document.path)

            val outcome = library.trashDocument(documentId = document.id, confirmed = true)

            val done = assertIs<OperationOutcome.Done>(value = outcome)
            assertTrue(actual = done.pendingFolders.isEmpty())
            library.allFolders.forEach { folder ->
                assertTrue(actual = library.folders.paths(folder = folder).isEmpty())
            }
            assertTrue(actual = library.catalog.documents.isEmpty())
        }
    }
}
