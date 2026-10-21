package dms.application.sync

import dms.application.TestLibrary
import dms.application.sync.model.SyncOutcome
import dms.domain.catalog.CatalogUpdate
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull

class MergeAllFoldersTest {

    @Test
    fun `files missing anywhere are copied, conflicts are not, and the catalogs are merged`() {
        runTest {
            val library = TestLibrary()
            val opened = library.open()
            library.folders.put(folder = library.syncA, path = "inbox/x.pdf", content = "x")
            library.folders.put(folder = library.syncB, path = "inbox/y.pdf", content = "y")
            library.folders.put(folder = library.primary, path = "c.txt", content = "one")
            library.folders.put(folder = library.syncB, path = "c.txt", content = "two")
            val catalogued = TestLibrary.document(id = "d-y", path = "inbox/y.pdf", content = "y")
            val newer = opened.addDocument(document = catalogued)
            library.folders.catalogs[library.syncB.id] = assertIs<CatalogUpdate.Applied>(value = newer).catalog

            val outcome = assertIs<SyncOutcome.Completed>(value = library.mergeAllFolders())

            library.allFolders.forEach { folder ->
                assertEquals(expected = "x", actual = library.folders.content(folder = folder, path = "inbox/x.pdf"))
                assertEquals(expected = "y", actual = library.folders.content(folder = folder, path = "inbox/y.pdf"))
            }
            assertEquals(expected = null, actual = library.folders.content(folder = library.syncA, path = "c.txt"))
            assertEquals(expected = listOf(RelativePath.of(raw = "c.txt")), actual = outcome.report.conflicts)
            assertNotNull(actual = library.catalog.documents[catalogued.id])
        }
    }

    @Test
    fun `a document moved outside the tool is not turned into a duplicate`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val document = library.add(path = "letters/a.pdf", content = "alpha")
            library.folders.move(folder = library.syncA, from = document.path, to = RelativePath.of(raw = "old/a.pdf"))

            val outcome = assertIs<SyncOutcome.Completed>(value = library.mergeAllFolders())

            assertEquals(expected = setOf("old/a.pdf"), actual = library.folders.paths(folder = library.syncA))
            assertEquals(expected = setOf("letters/a.pdf"), actual = library.folders.paths(folder = library.primary))
            assertEquals(
                expected = listOf(RelativePath.of(raw = "old/a.pdf")),
                actual = outcome.report.leftAlone[library.syncA.id],
            )
        }
    }
}
