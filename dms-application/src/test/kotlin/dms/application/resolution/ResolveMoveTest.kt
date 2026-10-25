package dms.application.resolution

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.resolution.model.MoveDecision
import dms.domain.change.DetectedChange
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull

class ResolveMoveTest {

    private val moved = RelativePathModel.of(raw = "archive/a.pdf")

    @Test
    fun `an applied move reaches every folder and the catalog`() {
        runTest {
            val library = TestLibrary()
            val change = movedInSyncFolder(library = library)

            assertIs<OperationOutcome.Done>(value = library.resolveMove(change = change, decision = MoveDecision.APPLY))

            library.allFolders.forEach { folder ->
                assertEquals(expected = setOf(moved.value), actual = library.folders.paths(folder = folder))
            }
            assertEquals(expected = moved, actual = library.catalog.documents.getValue(key = change.documentId).path)
        }
    }

    @Test
    fun `an undone move puts the file back where the catalog has it`() {
        runTest {
            val library = TestLibrary()
            val change = movedInSyncFolder(library = library)

            assertIs<OperationOutcome.Done>(value = library.resolveMove(change = change, decision = MoveDecision.UNDO))

            assertEquals(expected = setOf("letters/a.pdf"), actual = library.folders.paths(folder = library.syncA))
            assertEquals(
                expected = change.previousPath,
                actual = library.catalog.documents.getValue(key = change.documentId).path,
            )
        }
    }

    private suspend fun movedInSyncFolder(library: TestLibrary): DetectedChange.MovedFile {
        library.open()
        val document = library.add(path = "letters/a.pdf", content = "alpha")
        library.folders.move(folder = library.syncA, from = document.path, to = moved)
        val scan = assertNotNull(actual = library.scanned())
        return scan.changes.filterIsInstance<DetectedChange.MovedFile>().single()
    }
}
