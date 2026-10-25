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

class TrashNewFileTest {

    @Test
    fun `a file found on one side only goes to the trash there, and only when confirmed`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.syncA, path = "inbox/stray.pdf", content = "stray")
            val scan = assertNotNull(actual = library.scanned())
            val newFile = scan.changes.filterIsInstance<DetectedChange.NewFile>().single()

            val refused = library.trashNewFile(change = newFile, confirmed = false)
            assertTrue(actual = library.folders.trashed.isEmpty())
            val outcome = library.trashNewFile(change = newFile, confirmed = true)

            assertIs<OperationOutcome.NotConfirmed>(value = refused)
            assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(
                expected = listOf(library.syncA.id to RelativePathModel.of(raw = "inbox/stray.pdf")),
                actual = library.folders.trashed,
            )
        }
    }
}
