package dms.application.library

import dms.application.TestLibrary
import dms.application.library.model.ScanProgress
import dms.application.port.state.model.ActivityKind
import dms.domain.change.DetectedChange
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class ScanLibraryTest {

    @Test
    fun `progress names each folder and counts its files`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = "a.pdf", content = "alpha")
            library.folders.put(folder = library.primary, path = "b.pdf", content = "beta")
            val reported = mutableListOf<ScanProgress>()

            library.scanLibrary(onProgress = { reported += it })

            val primary = reported.filter { it.folderNumber == 1 }
            assertEquals(expected = 3, actual = reported.last().folderCount)
            assertEquals(expected = listOf(null, 2, 2, 2), actual = primary.map { it.total })
            assertEquals(expected = listOf(0, 0, 1, 2), actual = primary.map { it.done })
            assertEquals(expected = RelativePath.of(raw = "b.pdf"), actual = primary.last().current)
        }
    }

    @Test
    fun `without adding, a scan changes nothing`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = "a.pdf", content = "alpha")

            val scan = assertNotNull(actual = library.scanLibrary())

            assertTrue(actual = library.catalog.documents.isEmpty())
            assertEquals(expected = 1, actual = scan.changes.filterIsInstance<DetectedChange.NewFile>().size)
        }
    }

    @Test
    fun `adding catalogs new files of the primary and leaves the rest to the dashboard`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.folders.put(folder = library.primary, path = "letters/a.pdf", content = "alpha")
            library.folders.put(folder = library.primary, path = "copy-of-a.pdf", content = "alpha")
            library.folders.put(folder = library.syncA, path = "inbox/scan.pdf", content = "scan")

            val scan = assertNotNull(actual = library.scanLibrary(addNewFiles = true))

            val paths = library.catalog.documents.values.map { it.path.value }
            assertEquals(expected = 1, actual = paths.size)
            assertTrue(actual = paths.single() in setOf("letters/a.pdf", "copy-of-a.pdf"))
            val newFiles = scan.changes.filterIsInstance<DetectedChange.NewFile>().map { it.path.value }.toSet()
            assertTrue(actual = "inbox/scan.pdf" in newFiles)
            assertEquals(
                expected = listOf(ActivityKind.DOCUMENT_ADDED),
                actual = library.activityLog.recent(limit = 10).map { it.kind },
            )
        }
    }
}
