package dms.pdf

import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.pdf.model.PdfMetadataModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.pdf.io.PdfFiles
import dms.pdf.support.TestPdfs
import kotlinx.coroutines.test.runTest
import org.apache.pdfbox.text.PDFTextStripper
import java.nio.file.Files
import java.nio.file.Path
import java.time.LocalDate
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class PdfBoxEditorTest {

    private val root: Path = Files.createTempDirectory("dms-pdf-")
    private val folder = TestPdfs.folder(root = root)
    private val editor = PdfBoxEditor(locations = TestPdfs.locations(root = root))

    @AfterTest
    fun deleteTemp() {
        root.toFile().deleteRecursively()
    }

    private fun pageTexts(file: Path): List<String> {
        return PdfFiles.load(file = file).use { pdf ->
            (1..pdf.numberOfPages).map { number ->
                PDFTextStripper().apply {
                    startPage = number
                    endPage = number
                }.getText(pdf).trim()
            }
        }
    }

    @Test
    fun `pages are reordered, turned and left out in a new file, and the source stays as it was`() {
        runTest {
            TestPdfs.write(file = root.resolve("scan.pdf"), pages = listOf("first page", "second page", "third page"))
            val before = Files.readAllBytes(root.resolve("scan.pdf"))

            val written = editor.rewrite(
                folder = folder,
                source = TestPdfs.path(raw = "scan.pdf"),
                pages = listOf(PageChangeModel(page = 2, rotation = 90), PageChangeModel(page = 0)),
                target = TestPdfs.path(raw = "out/edited.pdf"),
            )

            assertIs<StorageResult.Success<*>>(value = written)
            assertEquals(
                expected = listOf("third page", "first page"),
                actual = pageTexts(file = root.resolve("out/edited.pdf")),
            )
            PdfFiles.load(file = root.resolve("out/edited.pdf")).use { pdf ->
                assertEquals(expected = listOf(90, 0), actual = pdf.pages.map { it.rotation })
            }
            assertContentEquals(expected = before, actual = Files.readAllBytes(root.resolve("scan.pdf")))
            assertEquals(
                expected = StorageResult.Success(value = 3),
                actual = editor.pageCount(folder = folder, path = TestPdfs.path(raw = "scan.pdf")),
            )
        }
    }

    @Test
    fun `merging keeps the order of the sources`() {
        runTest {
            TestPdfs.write(file = root.resolve("a.pdf"), pages = listOf("alpha one", "alpha two"))
            TestPdfs.write(file = root.resolve("b.pdf"), pages = listOf("beta one"))

            editor.merge(
                folder = folder,
                sources = listOf(TestPdfs.path(raw = "b.pdf"), TestPdfs.path(raw = "a.pdf")),
                target = TestPdfs.path(raw = "merged.pdf"),
            )

            assertEquals(
                expected = listOf("beta one", "alpha one", "alpha two"),
                actual = pageTexts(file = root.resolve("merged.pdf")),
            )
        }
    }

    @Test
    fun `a write never replaces an existing file and leaves the original untouched`() {
        runTest {
            TestPdfs.write(file = root.resolve("scan.pdf"), pages = listOf("page"))
            Files.writeString(root.resolve("taken.pdf"), "someone else's file")

            val written = editor.rewrite(
                folder = folder,
                source = TestPdfs.path(raw = "scan.pdf"),
                pages = listOf(PageChangeModel(page = 0)),
                target = TestPdfs.path(raw = "taken.pdf"),
            )
            val broken = editor.rewrite(
                folder = folder,
                source = TestPdfs.path(raw = "taken.pdf"),
                pages = listOf(PageChangeModel(page = 0)),
                target = TestPdfs.path(raw = "new.pdf"),
            )

            assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = (written as StorageResult.Failure).reason)
            assertEquals(expected = "someone else's file", actual = Files.readString(root.resolve("taken.pdf")))
            assertEquals(
                expected = StorageFailure.UNSUPPORTED_FORMAT,
                actual = (broken as StorageResult.Failure).reason,
            )
            assertTrue(
                actual = Files.list(root).use { files ->
                    files.noneMatch { it.fileName.toString().startsWith(prefix = ".dms-tmp-") }
                },
            )
        }
    }

    @Test
    fun `an invisible text layer makes a scan searchable`() {
        runTest {
            TestPdfs.write(file = root.resolve("scan.pdf"), pages = listOf(null, null))
            val scan = TestPdfs.path(raw = "scan.pdf")
            val before = editor.hasTextLayer(folder = folder, path = scan)

            editor.addTextLayer(
                folder = folder,
                source = scan,
                pages = listOf("Sample Energy invoice for Müller", "Total amount due"),
                target = TestPdfs.path(raw = "searchable.pdf"),
            )

            assertEquals(expected = StorageResult.Success(value = false), actual = before)
            assertEquals(
                expected = StorageResult.Success(value = true),
                actual = editor.hasTextLayer(folder = folder, path = TestPdfs.path(raw = "searchable.pdf")),
            )
            assertEquals(
                expected = listOf("Sample Energy invoice for Müller", "Total amount due"),
                actual = pageTexts(file = root.resolve("searchable.pdf")),
            )
        }
    }

    @Test
    fun `embedded metadata reads back`() {
        runTest {
            TestPdfs.write(file = root.resolve("bill.pdf"), pages = listOf("bill"))
            val metadata = PdfMetadataModel(
                title = "Power bill",
                author = "Sample Energy",
                subject = "Bills / Power",
                keywords = listOf("Tax", "Home"),
                date = LocalDate.of(2024, 3, 15),
            )

            editor.writeMetadata(
                folder = folder,
                source = TestPdfs.path(raw = "bill.pdf"),
                metadata = metadata,
                target = TestPdfs.path(raw = "tagged.pdf"),
            )

            assertEquals(
                expected = StorageResult.Success(value = metadata),
                actual = editor.readMetadata(folder = folder, path = TestPdfs.path(raw = "tagged.pdf")),
            )
            PdfFiles.load(file = root.resolve("tagged.pdf")).use { pdf ->
                assertNotNull(actual = pdf.documentCatalog.metadata)
            }
        }
    }

    @Test
    fun `a page renders as a small picture`() {
        runTest {
            TestPdfs.write(file = root.resolve("scan.pdf"), pages = listOf("page"))

            val image = assertNotNull(
                actual = editor.renderPage(
                    folder = folder,
                    path = TestPdfs.path(raw = "scan.pdf"),
                    page = 0,
                    maxSide = 120,
                ),
            )

            assertEquals(expected = "image/png", actual = image.mimeType)
            assertEquals(
                expected = null,
                actual = editor.renderPage(
                    folder = folder,
                    path = TestPdfs.path(raw = "scan.pdf"),
                    page = 5,
                    maxSide = 120,
                ),
            )
        }
    }
}
