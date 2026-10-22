package dms.application.pdf

import dms.application.TestLibrary
import dms.application.pdf.model.SearchableOutcome
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class SearchablePdfTest {

    @Test
    fun `a scan gets the transcription as a text layer, one text per page`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1|p2")
            library.folders.write(folder = library.primary, documentId = scan.id, text = "first page\u000Csecond page")

            val outcome = library.makeSearchable(documentId = scan.id)

            assertIs<SearchableOutcome.Made>(value = outcome)
            assertEquals(
                expected = "p1#first page|p2#second page",
                actual = library.folders.content(folder = library.syncB, path = "scan.pdf"),
            )
        }
    }

    @Test
    fun `a PDF with text is left alone, and one without transcription says so`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val searchable = library.add(path = "text.pdf", content = "p1#text")
            val scan = library.add(path = "scan.pdf", content = "p1")
            val photo = library.add(path = "photo.jpg", content = "jpg")

            assertEquals(
                expected = SearchableOutcome.AlreadySearchable,
                actual = library.makeSearchable(documentId = searchable.id),
            )
            assertEquals(
                expected = SearchableOutcome.NoTranscription,
                actual = library.makeSearchable(documentId = scan.id),
            )
            assertEquals(expected = SearchableOutcome.NotPdf, actual = library.makeSearchable(documentId = photo.id))
            assertEquals(
                expected = "p1#text",
                actual = library.folders.content(folder = library.primary, path = "text.pdf"),
            )
        }
    }

    @Test
    fun `precise OCR needs the program installed`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val scan = library.add(path = "scan.pdf", content = "p1")

            assertEquals(
                expected = SearchableOutcome.OcrUnavailable,
                actual = library.makeSearchable(documentId = scan.id, precise = true),
            )
            library.ocr.installed = true
            assertIs<SearchableOutcome.Made>(value = library.makeSearchable(documentId = scan.id, precise = true))
            assertEquals(
                expected = "p1#ocr",
                actual = library.folders.content(folder = library.primary, path = "scan.pdf"),
            )
        }
    }

    @Test
    fun `the batch skips PDFs that have text and finds the scans first`() {
        runTest {
            val library = TestLibrary()
            library.open()
            library.add(path = "a-text.pdf", content = "p1#text")
            val scan = library.add(path = "b-scan.pdf", content = "p1")
            library.folders.write(folder = library.primary, documentId = scan.id, text = "words")

            val before = library.scansWithoutText()
            val progress = library.makeAllSearchable().toList()

            assertEquals(expected = listOf(scan.id), actual = before)
            assertEquals(
                expected = listOf(SearchableOutcome.AlreadySearchable::class, SearchableOutcome.Made::class),
                actual = progress.map { it.outcome::class },
            )
            assertEquals(expected = emptyList(), actual = library.scansWithoutText())
        }
    }

    @Test
    fun `metadata is only embedded while the library's setting is on`() {
        runTest {
            val library = TestLibrary()
            library.open()
            val bill = library.add(path = "bill.pdf", content = "p1")
            library.store.commit {
                it.updateMetadata(
                    documentId = bill.id,
                    metadata = dms.domain.document.model.DocumentMetadataModel(title = "Power bill"),
                )
            }

            library.embedPdfMetadata(documentId = bill.id)
            assertEquals(expected = "p1", actual = library.folders.content(folder = library.primary, path = "bill.pdf"))

            library.changeSettings(subject = "embed") { it.withLayout(layout = it.layout.copy(embedMetadata = true)) }
            library.embedPdfMetadata(documentId = bill.id)

            assertEquals(
                expected = "p1;Power bill",
                actual = library.folders.content(folder = library.syncA, path = "bill.pdf"),
            )
        }
    }
}
