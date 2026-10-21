package dms.ai.read

import dms.application.port.ai.model.DocumentContent
import dms.application.port.state.model.AiSettings
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import org.apache.pdfbox.pdmodel.PDPageContentStream
import org.apache.pdfbox.pdmodel.font.PDType1Font
import org.apache.pdfbox.pdmodel.font.Standard14Fonts
import org.apache.poi.xwpf.usermodel.XWPFDocument
import java.awt.image.BufferedImage
import java.nio.file.Files
import java.nio.file.Path
import java.time.Instant
import javax.imageio.ImageIO
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

class FileContentReaderTest {

    private val root: Path = Files.createTempDirectory("dms-ai-")
    private val folder = LibraryFolder(
        id = LibraryFolderId(value = "primary"),
        displayName = "Primary",
        location = FolderLocation(value = root.toString()),
        role = FolderRole.PRIMARY,
    )
    private var settings = AiSettings()
    private val reader = FileContentReader(
        locate = { _, path -> root.resolve(path.value).takeIf { Files.exists(it) } },
        settings = { settings },
    )
    private val sampleText = "Sample invoice number 4711 for synthetic test services. ".repeat(n = 4)

    @AfterTest
    fun deleteTemp() {
        root.toFile().deleteRecursively()
    }

    @Test
    fun `a PDF with a text layer is read as text`() {
        runTest {
            writePdf(name = "a.pdf", text = sampleText)

            val content = reader.read(folder = folder, document = document(name = "a.pdf"))

            assertTrue(actual = "Sample invoice number 4711" in assertIs<DocumentContent.Text>(value = content).text)
        }
    }

    @Test
    fun `the text for search is the whole text layer, and nothing for a scan or an image`() {
        runTest {
            writePdf(name = "a.pdf", text = sampleText)
            writePdf(name = "scan.pdf", text = null)

            val text = reader.text(folder = folder, document = document(name = "a.pdf"))
            val scan = reader.text(folder = folder, document = document(name = "scan.pdf"))
            val missing = reader.text(folder = folder, document = document(name = "gone.png"))

            assertTrue(actual = text.orEmpty().contains(other = "Sample invoice number 4711"))
            assertNull(actual = scan)
            assertNull(actual = missing)
        }
    }

    @Test
    fun `a PDF without text, or with vision preferred, is sent as page images`() {
        runTest {
            writePdf(name = "scan.pdf", text = null)
            writePdf(name = "text.pdf", text = sampleText)
            val scan = reader.read(folder = folder, document = document(name = "scan.pdf"))
            settings = AiSettings(preferVisionForPdfs = true)
            val preferred = reader.read(folder = folder, document = document(name = "text.pdf"))

            val page = assertIs<DocumentContent.PageImages>(value = scan).pages.single()
            assertEquals(expected = "image/jpeg", actual = page.mimeType)
            assertIs<DocumentContent.PageImages>(value = preferred)
        }
    }

    @Test
    fun `an image is downscaled for the model`() {
        runTest {
            val photo = BufferedImage(4000, 2000, BufferedImage.TYPE_INT_ARGB)
            ImageIO.write(photo, "png", root.resolve("photo.png").toFile())

            val content = reader.read(folder = folder, document = document(name = "photo.png"))

            val bytes = assertIs<DocumentContent.PageImages>(value = content).pages.single().bytes
            val image = ImageIO.read(bytes.inputStream())
            assertEquals(expected = VisionImages.MAX_SIDE, actual = image.width)
            assertEquals(expected = VisionImages.MAX_SIDE / 2, actual = image.height)
        }
    }

    @Test
    fun `a Word file is read as text`() {
        runTest {
            XWPFDocument().use { word ->
                word.createParagraph().createRun().setText("Synthetic contract text")
                Files.newOutputStream(root.resolve("contract.docx")).use { word.write(it) }
            }

            val content = reader.read(folder = folder, document = document(name = "contract.docx"))

            val text = assertIs<DocumentContent.Text>(value = content).text
            assertEquals(expected = "Synthetic contract text", actual = text)
        }
    }

    @Test
    fun `a missing file or another kind is not readable`() {
        runTest {
            Files.writeString(root.resolve("song.mp3"), "x")

            val missing = reader.read(folder = folder, document = document(name = "gone.pdf"))
            val song = reader.read(folder = folder, document = document(name = "song.mp3"))

            assertIs<DocumentContent.NotReadable>(value = missing)
            assertIs<DocumentContent.NotReadable>(value = song)
        }
    }

    private fun writePdf(name: String, text: String?) {
        PDDocument().use { pdf ->
            val page = PDPage()
            pdf.addPage(page)
            if (text != null) {
                PDPageContentStream(pdf, page).use { stream ->
                    stream.beginText()
                    stream.setFont(PDType1Font(Standard14Fonts.FontName.HELVETICA), 10f)
                    stream.setLeading(14f)
                    stream.newLineAtOffset(40f, 700f)
                    text.split(". ").forEach { line ->
                        stream.showText(line)
                        stream.newLine()
                    }
                    stream.endText()
                }
            }
            pdf.save(root.resolve(name).toFile())
        }
    }

    private fun document(name: String): Document {
        return Document(
            id = DocumentId(value = name),
            path = RelativePath.of(raw = name),
            contentHash = ContentHash(value = "a".repeat(n = 64)),
            size = 1,
            addedAt = Instant.parse("2026-01-01T00:00:00Z"),
        )
    }
}
