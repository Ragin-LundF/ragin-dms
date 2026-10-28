package dms.application.pdf.rebuild

import dms.application.TestLibrary
import dms.application.common.OperationOutcome
import dms.application.fake.storage.FakeFolders
import dms.application.pdf.rebuild.model.RebuildChoice
import dms.application.pdf.rebuild.model.RebuildOutcome
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.pdf.model.RebuiltFileModel
import dms.application.port.state.model.ActivityKind
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rebuild.FontFamily
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.LayoutLineModel
import dms.domain.rebuild.model.PageLayoutModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class KeepRebuildTest {

    private val library = TestLibrary()

    @Test
    fun `keeping the original changes nothing and drops the rebuild`() {
        runTest {
            val (scan, rebuilt) = rebuiltScan()
            val before = library.catalog

            val outcome = library.keepRebuild(
                documentId = scan.id,
                rebuilt = rebuilt,
                choice = RebuildChoice.KEEP_ORIGINAL,
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            assertEquals(expected = before, actual = library.catalog)
            assertEquals(expected = "p1", actual = library.folders.content(folder = library.primary, path = SCAN))
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    @Test
    fun `using the rebuild replaces the file in every folder, trashes the scan and logs it`() {
        runTest {
            val (scan, rebuilt) = rebuiltScan()

            val outcome = library.keepRebuild(
                documentId = scan.id,
                rebuilt = rebuilt,
                choice = RebuildChoice.REPLACE_WITH_REBUILD,
            )

            assertIs<OperationOutcome.Done>(value = outcome)
            library.allFolders.forEach { folder ->
                assertEquals(expected = "Sample line", actual = library.folders.content(folder = folder, path = SCAN))
            }
            val document = assertNotNull(actual = library.catalog.documents[scan.id])
            assertEquals(expected = "Sample line".length.toLong(), actual = document.size)
            assertTrue(actual = library.primary.id to scan.path in library.folders.trashed)
            val entry = library.activityLog.recent(limit = 1).single()
            assertEquals(expected = ActivityKind.PDF_EDITED, actual = entry.kind)
            assertEquals(expected = "Rebuilt: letters/scan.pdf", actual = entry.subject)
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    @Test
    fun `keeping both adds the rebuild next to the original with its category, tags and fields`() {
        runTest {
            val (scan, rebuilt) = rebuiltScan()

            val outcome = library.keepRebuild(documentId = scan.id, rebuilt = rebuilt, choice = RebuildChoice.KEEP_BOTH)

            assertIs<OperationOutcome.Done>(value = outcome)
            val copyPath = RelativePathModel.of(raw = "letters/scan (rebuilt).pdf")
            val copy = assertNotNull(actual = library.catalog.findByPath(path = copyPath))
            assertEquals(expected = setOf(TagNameModel.of(raw = "Tax")), actual = copy.tags)
            assertEquals(expected = scan.fields, actual = copy.fields)
            assertTrue(actual = copy.id != scan.id)
            library.allFolders.forEach { folder ->
                assertEquals(expected = "p1", actual = library.folders.content(folder = folder, path = SCAN))
                assertEquals(
                    expected = "Sample line",
                    actual = library.folders.content(folder = folder, path = "letters/scan (rebuilt).pdf"),
                )
            }
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    @Test
    fun `a rebuild that cannot be placed stays for another choice`() {
        runTest {
            val (scan, rebuilt) = rebuiltScan()
            val before = library.catalog
            library.folders.unreachable += library.primary.id

            val outcome = library.keepRebuild(documentId = scan.id, rebuilt = rebuilt, choice = RebuildChoice.KEEP_BOTH)

            assertIs<OperationOutcome.StorageFailed>(value = outcome)
            assertEquals(expected = before, actual = library.catalog)
            assertEquals(expected = setOf(rebuilt.id), actual = library.rebuilder.waiting.keys)
        }
    }

    @Test
    fun `a scan that changed after the conversion is not replaced, and keeping both still works`() {
        runTest {
            val (scan, rebuilt) = rebuiltScan()
            library.folders.put(folder = library.primary, path = SCAN, content = "p1@90")
            library.store.commit {
                it.updateContent(documentId = scan.id, contentHash = FakeFolders.hashOf(content = "p1@90"), size = 5)
            }

            val replaced = library.keepRebuild(
                documentId = scan.id,
                rebuilt = rebuilt,
                choice = RebuildChoice.REPLACE_WITH_REBUILD,
            )

            assertEquals(expected = OperationOutcome.SourceChanged, actual = replaced)
            assertEquals(expected = "p1@90", actual = library.folders.content(folder = library.primary, path = SCAN))
            assertEquals(expected = setOf(rebuilt.id), actual = library.rebuilder.waiting.keys)
            val both = library.keepRebuild(documentId = scan.id, rebuilt = rebuilt, choice = RebuildChoice.KEEP_BOTH)
            assertIs<OperationOutcome.Done>(value = both)
        }
    }

    @Test
    fun `a rebuild whose document is gone is dropped, whatever the choice`() {
        runTest {
            val (scan, rebuilt) = rebuiltScan()
            library.store.commit { it.removeDocument(documentId = scan.id) }

            val outcome = library.keepRebuild(documentId = scan.id, rebuilt = rebuilt, choice = RebuildChoice.KEEP_BOTH)

            assertEquals(expected = OperationOutcome.NotFound, actual = outcome)
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    /** A tagged scan with one rebuilt page that reads "Sample line". */
    private suspend fun rebuiltScan(): Pair<DocumentModel, RebuiltFileModel> {
        library.open()
        val added = library.add(path = SCAN, content = "p1")
        library.store.commit { it.setTags(documentId = added.id, newTags = setOf(TagNameModel.of(raw = "Tax"))) }
        library.layoutReader.layouts[0] = LayoutOutcome.Read(
            layout = PageLayoutModel(
                lines = listOf(
                    LayoutLineModel(
                        box = LayoutBoxModel(left = 100.0, top = 100.0, right = 600.0, bottom = 130.0),
                        text = "Sample line",
                        bold = false,
                        family = FontFamily.SANS,
                        legible = true,
                    ),
                ),
                graphics = emptyList(),
            ),
        )
        val outcome = assertIs<RebuildOutcome.Rebuilt>(value = library.rebuildPdf(documentId = added.id))
        return assertNotNull(actual = library.catalog.documents[added.id]) to outcome.rebuilt
    }

    private companion object {
        const val SCAN = "letters/scan.pdf"
    }
}
