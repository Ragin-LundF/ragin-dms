package dms.application.sync

import dms.application.TestLibrary
import dms.application.sync.model.SyncActionKind
import dms.application.sync.model.SyncActionModel
import dms.application.sync.model.SyncOutcome
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class SyncFromPrimaryTest {

    @Test
    fun `a sync folder gets what it missed, keeps its own files and nothing is overwritten`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val moved = library.add(path = "letters/b.pdf", content = "beta")
            library.folders.unreachable += library.syncA.id
            library.add(path = "letters/a.pdf", content = "alpha")
            library.moveDocument(documentId = moved.id, newPath = RelativePathModel.of(raw = "archive/b.pdf"))
            library.folders.unreachable -= library.syncA.id
            library.folders.put(folder = library.syncA, path = "only-here.txt", content = "own")
            library.folders.put(folder = library.primary, path = "notes/n.txt", content = "one")
            library.folders.put(folder = library.syncA, path = "notes/n.txt", content = "two")

            val outcome = assertIs<SyncOutcome.Completed>(value = library.syncFromPrimary())

            val report = outcome.report
            val movedInSyncA = action(library = library, kind = SyncActionKind.MOVED, path = "archive/b.pdf")
            val copiedToSyncA = action(library = library, kind = SyncActionKind.COPIED, path = "letters/a.pdf")
            assertTrue(actual = movedInSyncA in report.actions)
            assertTrue(actual = copiedToSyncA in report.actions)
            assertEquals(expected = listOf(RelativePathModel.of(raw = "notes/n.txt")), actual = report.conflicts)
            assertEquals(
                expected = listOf(RelativePathModel.of(raw = "only-here.txt")),
                actual = report.leftAlone[library.syncA.id],
            )
            assertEquals(
                expected = setOf("archive/b.pdf", "letters/a.pdf", "notes/n.txt", "only-here.txt"),
                actual = library.folders.paths(folder = library.syncA),
            )
            assertEquals(
                expected = "two",
                actual = library.folders.content(folder = library.syncA, path = "notes/n.txt"),
            )
            assertEquals(expected = library.catalog, actual = library.folders.catalogs[library.syncA.id])
        }
    }

    @Test
    fun `an unreachable sync folder is reported`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.unreachable += library.syncB.id

            val outcome = assertIs<SyncOutcome.Completed>(value = library.syncFromPrimary())

            assertEquals(expected = setOf(library.syncB.id), actual = outcome.report.unreachableFolders)
        }
    }

    private fun action(library: TestLibrary, kind: SyncActionKind, path: String): SyncActionModel {
        return SyncActionModel(folderId = library.syncA.id, kind = kind, path = RelativePathModel.of(raw = path))
    }
}
