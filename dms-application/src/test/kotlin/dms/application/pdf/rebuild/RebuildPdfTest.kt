package dms.application.pdf.rebuild

import dms.application.TestLibrary
import dms.application.fake.storage.FakeFolders
import dms.application.pdf.rebuild.model.RebuildBlocker
import dms.application.pdf.rebuild.model.RebuildOutcome
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import dms.domain.rebuild.FontFamily
import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.LayoutLineModel
import dms.domain.rebuild.model.PageLayoutModel
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

class RebuildPdfTest {

    @Test
    fun `a page the AI cannot read in time stays as it was and the next page is still rebuilt`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "letters/scan.pdf", content = "p1|p2")
            library.layoutReader.layouts[0] = LayoutOutcome.Failure(reason = ClassificationFailure.TIMEOUT)
            library.layoutReader.layouts[1] = read(line(text = "Sample line"))
            val pages = mutableListOf<Int>()

            val outcome = library.rebuildPdf(documentId = scan.id) { page, _ -> pages += page }

            val rebuilt = assertIs<RebuildOutcome.Rebuilt>(value = outcome).rebuilt
            assertEquals(expected = "page 0 kept|Sample line", actual = library.rebuilder.waiting[rebuilt.id])
            assertEquals(expected = mapOf(KeepReason.PAGE_FAILED to 1), actual = rebuilt.pages[0].keptReasons)
            // Page 0, page 1, then writing the file (page == pages).
            assertEquals(expected = listOf(0, 1, 2), actual = pages)
            assertEquals(expected = scan.size, actual = outcome.originalSize)
            val original = library.folders.content(folder = library.primary, path = "letters/scan.pdf")
            assertEquals(expected = "p1|p2", actual = original)
        }
    }

    @Test
    fun `lost characters are read again, and a reading that does not fit leaves the line an image`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1")
            val fits = line(text = "Gr\u0000\u0000e aus K\u0000ln", top = 100.0)
            val differs = line(text = "Herr M\u0000ller", top = 200.0)
            library.layoutReader.layouts[0] = read(fits, differs)
            library.layoutReader.lines[fits.box.toString()] = LineOutcome.Read(text = " Grüße  aus Köln ")
            library.layoutReader.lines[differs.box.toString()] = LineOutcome.Read(text = "Herr Mueller")

            val outcome = assertIs<RebuildOutcome.Rebuilt>(value = library.rebuildPdf(documentId = scan.id))

            assertEquals(expected = "Grüße aus Köln/[image]", actual = library.rebuilder.waiting[outcome.rebuilt.id])
            val kept = outcome.rebuilt.pages[0].keptReasons
            assertEquals(expected = mapOf(KeepReason.LOST_CHARACTERS to 1), actual = kept)
        }
    }

    @Test
    fun `an AI that cannot read images stops the run and writes nothing`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1|p2")
            val noVision = LayoutOutcome.Failure(reason = ClassificationFailure.MODEL_CANNOT_READ_IMAGES)
            library.layoutReader.layouts[0] = noVision

            val outcome = library.rebuildPdf(documentId = scan.id)

            assertEquals(
                expected = RebuildOutcome.NotAvailable(reason = RebuildBlocker.MODEL_CANNOT_READ_IMAGES),
                actual = outcome,
            )
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    @Test
    fun `a PDF changed during the run is not written, since its pages would come from two versions`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1|p2")
            library.layoutReader.beforeLayout = { page ->
                if (page == 1) {
                    library.store.commit {
                        val changed = FakeFolders.hashOf(content = "p1")
                        it.updateContent(documentId = scan.id, contentHash = changed, size = 2)
                    }
                }
            }

            val outcome = library.rebuildPdf(documentId = scan.id)

            val expected = RebuildOutcome.NotAvailable(reason = RebuildBlocker.SOURCE_CHANGED)
            assertEquals(expected = expected, actual = outcome)
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    @Test
    fun `cancelling during a page rethrows and leaves no rebuilt file`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1|p2")
            val reading = CompletableDeferred<Unit>()
            library.layoutReader.beforeLayout = { page ->
                if (page == 1) {
                    reading.complete(value = Unit)
                    awaitCancellation()
                }
            }

            val run = async { library.rebuildPdf(documentId = scan.id) }
            reading.await()
            run.cancel()

            assertTrue(actual = runCatching { run.await() }.isFailure)
            assertTrue(actual = run.isCancelled)
            assertTrue(actual = library.rebuilder.waiting.isEmpty())
        }
    }

    @Test
    fun `only PDFs without a text layer can be rebuilt`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1")
            val searchable = library.add(path = "text.pdf", content = "p1#text")
            val photo = library.add(path = "photo.jpg", content = "jpg")

            assertTrue(actual = library.rebuildAvailability(documentId = scan.id))
            assertFalse(actual = library.rebuildAvailability(documentId = searchable.id))
            assertFalse(actual = library.rebuildAvailability(documentId = photo.id))
            assertEquals(
                expected = RebuildOutcome.NotAvailable(reason = RebuildBlocker.HAS_TEXT_LAYER),
                actual = library.rebuildPdf(documentId = searchable.id),
            )
        }
    }

    private fun read(vararg lines: LayoutLineModel): LayoutOutcome {
        return LayoutOutcome.Read(layout = PageLayoutModel(lines = lines.toList(), graphics = emptyList()))
    }

    private fun line(text: String, top: Double = 100.0): LayoutLineModel {
        return LayoutLineModel(
            box = LayoutBoxModel(left = 100.0, top = top, right = 600.0, bottom = top + 30.0),
            text = text,
            bold = false,
            family = FontFamily.SANS,
            legible = true,
        )
    }
}
