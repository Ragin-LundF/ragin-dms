package dms.ai.read

import dms.domain.library.FolderLocation
import dms.domain.library.FolderRole
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import kotlinx.coroutines.test.runTest
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import org.apache.poi.xwpf.usermodel.XWPFDocument
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.ImageIO
import kotlin.math.max
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class FilePreviewerTest {

    private val root: Path = Files.createTempDirectory("dms-preview-")
    private val folder = LibraryFolder(
        id = LibraryFolderId(value = "primary"),
        displayName = "Primary",
        location = FolderLocation(value = root.toString()),
        role = FolderRole.PRIMARY,
    )
    private val previewer = FilePreviewer(locate = { _, path -> root.resolve(path.value).takeIf { Files.exists(it) } })

    @AfterTest
    fun deleteTemp() {
        root.toFile().deleteRecursively()
    }

    @Test
    fun `a PDF shows its first page, scaled down`() {
        runTest {
            PDDocument().use { pdf ->
                pdf.addPage(PDPage())
                pdf.save(root.resolve("a.pdf").toFile())
            }

            val thumbnail = previewer.thumbnail(folder = folder, path = RelativePath.of(raw = "a.pdf"), maxSide = 120)

            val image = ImageIO.read(ByteArrayInputStream(assertNotNull(actual = thumbnail).bytes))
            assertEquals(expected = 120, actual = max(image.width, image.height))
        }
    }

    @Test
    fun `an image is scaled down to the thumbnail size`() {
        runTest {
            val photo = BufferedImage(800, 400, BufferedImage.TYPE_INT_RGB)
            ImageIO.write(photo, "png", root.resolve("photo.png").toFile())
            val path = RelativePath.of(raw = "photo.png")

            val thumbnail = previewer.thumbnail(folder = folder, path = path, maxSide = 200)

            val image = ImageIO.read(ByteArrayInputStream(assertNotNull(actual = thumbnail).bytes))
            assertEquals(expected = 200, actual = image.width)
            assertEquals(expected = 100, actual = image.height)
        }
    }

    @Test
    fun `office files, missing files and broken files have no thumbnail`() {
        runTest {
            XWPFDocument().use { word ->
                Files.newOutputStream(root.resolve("letter.docx")).use { word.write(it) }
            }
            Files.writeString(root.resolve("broken.pdf"), "not a pdf")

            listOf("letter.docx", "missing.pdf", "broken.pdf").forEach { name ->
                val thumbnail = previewer.thumbnail(folder = folder, path = RelativePath.of(raw = name), maxSide = 100)
                assertNull(actual = thumbnail, message = name)
            }
        }
    }
}
