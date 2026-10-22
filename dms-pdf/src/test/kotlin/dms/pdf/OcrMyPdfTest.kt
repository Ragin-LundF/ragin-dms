package dms.pdf

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.pdf.ocr.OcrMyPdf
import dms.pdf.support.TestPdfs
import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

/** The OCR program is replaced by a recorded command; it never needs to be installed. */
class OcrMyPdfTest {

    private val root: Path = Files.createTempDirectory("dms-ocr-")
    private val folder = TestPdfs.folder(root = root)
    private val commands = mutableListOf<List<String>>()

    @AfterTest
    fun deleteTemp() {
        root.toFile().deleteRecursively()
    }

    @Test
    fun `the program is available when it answers to --version`() {
        runTest {
            val installed = OcrMyPdf(locations = TestPdfs.locations(root = root), runner = { 0 })
            val missing = OcrMyPdf(locations = TestPdfs.locations(root = root), runner = { null })

            assertTrue(actual = installed.available())
            assertFalse(actual = missing.available())
        }
    }

    @Test
    fun `the searchable copy is moved to the target`() {
        runTest {
            TestPdfs.write(file = root.resolve("scan.pdf"), pages = listOf(null))
            val ocr = OcrMyPdf(locations = TestPdfs.locations(root = root), runner = { command ->
                commands += command
                Files.copy(Path.of(command[command.size - 2]), Path.of(command.last()))
                0
            })

            val written = ocr.ocr(
                folder = folder,
                source = TestPdfs.path(raw = "scan.pdf"),
                target = TestPdfs.path(raw = "ocr.pdf"),
            )

            assertIs<StorageResult.Success<*>>(value = written)
            assertTrue(actual = Files.exists(root.resolve("ocr.pdf")))
            assertEquals(
                expected = listOf("ocrmypdf", "--skip-text", "--quiet"),
                actual = commands.single().take(n = 3),
            )
        }
    }

    @Test
    fun `a failing program leaves nothing behind`() {
        runTest {
            TestPdfs.write(file = root.resolve("scan.pdf"), pages = listOf(null))
            val ocr = OcrMyPdf(locations = TestPdfs.locations(root = root), runner = { 2 })

            val written = ocr.ocr(
                folder = folder,
                source = TestPdfs.path(raw = "scan.pdf"),
                target = TestPdfs.path(raw = "ocr.pdf"),
            )

            assertEquals(expected = StorageFailure.IO_ERROR, actual = (written as StorageResult.Failure).reason)
            assertEquals(
                expected = listOf("scan.pdf"),
                actual = Files.list(root).use { files -> files.map { it.fileName.toString() }.toList() },
            )
        }
    }
}
