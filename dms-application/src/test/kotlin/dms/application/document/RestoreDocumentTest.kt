package dms.application.document

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.domain.change.DetectedChange
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull

class RestoreDocumentTest {

    @Test
    fun `a file deleted outside the tool is restored from a folder that still has it`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.moveToTrash(folder = library.primary, path = document.path)
            library.folders.moveToTrash(folder = library.syncA, path = document.path)
            val scan = assertNotNull(actual = library.scanLibrary())
            val missing = scan.changes.filterIsInstance<DetectedChange.MissingFile>().single()

            val outcome = library.restoreDocument(change = missing)

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(
                    expected = "alpha",
                    actual = library.folders.content(folder = folder, path = "letters/a.pdf"),
                )
            }
        }
    }
}
