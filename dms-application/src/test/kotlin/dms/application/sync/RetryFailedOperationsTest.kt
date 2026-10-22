package dms.application.sync

import dms.application.TestLibrary
import dms.application.sync.model.RetryReportModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class RetryFailedOperationsTest {

    @Test
    fun `operations for a folder that is back are applied in order`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id
            library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.unreachable -= library.syncB.id

            val report = library.retryFailedOperations()

            assertEquals(expected = RetryReportModel(succeeded = 2), actual = report)
            assertEquals(
                expected = "alpha",
                actual = library.folders.content(folder = library.syncB, path = "letters/a.pdf"),
            )
            assertEquals(expected = library.catalog, actual = library.folders.catalogs[library.syncB.id])
            assertTrue(actual = library.journal.pending().isEmpty())
        }
    }

    @Test
    fun `an operation whose file is gone is dropped and the rest continue`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            library.trashDocument(documentId = document.id, confirmed = true)
            library.folders.unreachable -= library.syncB.id

            val report = library.retryFailedOperations()

            assertEquals(expected = RetryReportModel(succeeded = 3, dropped = 1), actual = report)
            assertTrue(actual = library.folders.paths(folder = library.syncB).isEmpty())
            assertTrue(actual = library.journal.pending().isEmpty())
        }
    }

    @Test
    fun `operations for a folder that is still offline stay in the journal`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id
            library.add(path = "letters/a.pdf", content = "alpha")

            val report = library.retryFailedOperations()

            assertEquals(expected = RetryReportModel(stillPending = 2), actual = report)
            assertEquals(expected = 2, actual = library.journal.pending().size)
        }
    }
}
