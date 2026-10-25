package dms.pdf.io

import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.pdf.support.TestPdfs
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import java.nio.file.Files
import java.nio.file.Path
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

class PdfFilesTest {

    private val root: Path = Files.createTempDirectory("dms-pdf-files-")

    @AfterTest
    fun deleteTemp() {
        root.toFile().deleteRecursively()
    }

    @Test
    fun `a file that appears before the move is not replaced`() {
        val target = root.resolve("scan.pdf")
        Files.writeString(target, "arrived meanwhile")
        val temp = PdfFiles.tempFor(target = target)
        Files.writeString(temp, "written")

        val result = PdfFiles.moveWithoutReplacing(temp = temp, target = target)

        assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = (result as? StorageResult.Failure)?.reason)
        assertEquals(expected = "arrived meanwhile", actual = Files.readString(target))
    }

    @Test
    fun `saving onto a name that differs only by case fails and keeps the existing file`() {
        val existing = root.resolve("SCAN.pdf")
        Files.writeString(existing, "existing")

        val result = PDDocument().use { pdf ->
            pdf.addPage(PDPage())
            PdfFiles.save(document = pdf, target = root.resolve("scan.pdf"))
        }

        assertEquals(expected = StorageFailure.ALREADY_EXISTS, actual = (result as? StorageResult.Failure)?.reason)
        assertEquals(expected = "existing", actual = Files.readString(existing))
        assertEquals(expected = setOf("SCAN.pdf"), actual = namesIn(directory = root))
    }

    @Test
    fun `a saved file is placed under its name and leaves no temporary file`() {
        TestPdfs.write(file = root.resolve("source.pdf"), pages = listOf("page"))

        val result = PdfFiles.load(file = root.resolve("source.pdf")).use { pdf ->
            PdfFiles.save(document = pdf, target = root.resolve("out/copy.pdf"))
        }

        assertIs<StorageResult.Success<*>>(value = result)
        assertEquals(expected = setOf("copy.pdf"), actual = namesIn(directory = root.resolve("out")))
    }

    private fun namesIn(directory: Path): Set<String> {
        return Files.newDirectoryStream(directory).use { entries -> entries.map { it.fileName.toString() }.toSet() }
    }
}
