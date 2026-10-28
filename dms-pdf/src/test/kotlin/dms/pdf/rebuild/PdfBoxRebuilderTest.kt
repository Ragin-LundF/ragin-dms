package dms.pdf.rebuild

import dms.application.port.pdf.model.PageInput
import dms.application.port.pdf.model.PreparedPageModel
import dms.application.port.pdf.model.RebuiltFileModel
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.ContentHashModel
import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.RegionKind
import dms.domain.rebuild.model.LayoutLineModel
import dms.domain.rebuild.model.PageLayoutModel
import dms.pdf.io.PdfFiles
import dms.pdf.support.TestPdfs
import dms.pdf.support.TestScans
import kotlinx.coroutines.test.runTest
import org.apache.pdfbox.cos.COSName
import org.apache.pdfbox.pdmodel.graphics.image.PDImageXObject
import org.apache.pdfbox.rendering.ImageType
import org.apache.pdfbox.rendering.PDFRenderer
import org.apache.pdfbox.text.PDFTextStripper
import java.awt.Color
import java.awt.image.BufferedImage
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.exists
import kotlin.math.abs
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class PdfBoxRebuilderTest {

    private val root: Path = Files.createTempDirectory("dms-rebuild-")
    private val work: Path = root.resolve("work")
    private val folder = TestPdfs.folder(root = root)
    private val rebuilder = PdfBoxRebuilder(locations = TestPdfs.locations(root = root), workDirectory = work)

    @AfterTest
    fun deleteTemp() {
        root.toFile().deleteRecursively()
    }

    @Test
    fun `a page turned by two degrees is straightened and its lines become text with umlauts`() {
        runTest {
            val scan = TestScans()
            val lines = listOf(
                "Rechnung Nr. 4711 für Jörg Müller",
                "Beispielstraße 12 in 12345 Musterstadt",
                "Leistung im Zeitraum Januar bis März",
                "Gesamtbetrag 123,45 € inklusive Steuer",
                "Zahlbar bis Ende des Monats ohne Abzug",
                "Vielen Dank für Ihren Auftrag",
            ).mapIndexed { i, text -> scan.text(text = text, x = 250, y = 250 + i * 120) }
            scan.writePdf(file = root.resolve("scan.pdf"), page = scan.turned(degrees = 2.0))

            val prepared = assertIs<StorageResult.Success<PreparedPageModel>>(value = prepare()).value
            val rebuilt = rebuild(input = PageInput.Read(prepared = prepared, layout = layoutOf(lines = lines)))

            assertTrue(actual = abs(prepared.angle - 2.0) <= ANGLE_TOLERANCE, message = "angle ${prepared.angle}")
            val page = rebuilt.pages.single()
            assertEquals(expected = 6, actual = page.linesAsText, message = "kept ${page.keptReasons}")
            val text = textOf(rebuilt = rebuilt)
            assertTrue(actual = "für Jörg Müller" in text, message = text)
            assertTrue(actual = "123,45 €" in text, message = text)
        }
    }

    @Test
    fun `a grey but legible line becomes text through the faint mask`() {
        runTest {
            val scan = TestScans()
            val dark = scan.text(text = "Kundennummer 99887766 Beispiel", x = 150, y = 300)
            val grey = scan.text(text = "Seite 1 von 1 Beispieltext", x = 150, y = 450, gray = GREY_PRINT)
            scan.writePdf(file = root.resolve("scan.pdf"))

            val rebuilt = rebuild(input = read(layout = layoutOf(lines = listOf(dark, grey))))

            val page = rebuilt.pages.single()
            assertEquals(expected = 2, actual = page.linesAsText, message = "${page.keptReasons}")
            assertTrue(actual = "Seite 1 von 1" in textOf(rebuilt = rebuilt))
        }
    }

    @Test
    fun `a reading shorter than the ink stays an image as a mismatch`() {
        runTest {
            val scan = TestScans()
            val invented = scan.text(text = "Gesamtbetrag der Rechnung 123,45", x = 150, y = 300, read = "Gesamt 5")
            scan.writePdf(file = root.resolve("scan.pdf"))

            val rebuilt = rebuild(input = read(layout = layoutOf(lines = listOf(invented))))

            val page = rebuilt.pages.single()
            assertEquals(expected = mapOf(KeepReason.TEXT_MISMATCH to 1), actual = page.keptReasons)
            assertFalse(actual = "Gesamt 5" in textOf(rebuilt = rebuilt))
            assertTrue(actual = page.regions.any { it.kind == RegionKind.IMAGE })
        }
    }

    @Test
    fun `a faint line the AI is not sure about stays an image`() {
        runTest {
            val scan = TestScans()
            val faint = scan.text(text = "Kaum lesbarer Vermerk", x = 150, y = 300, gray = GREY_PRINT, legible = false)
            scan.writePdf(file = root.resolve("scan.pdf"))

            val rebuilt = rebuild(input = read(layout = layoutOf(lines = listOf(faint))))

            assertEquals(expected = mapOf(KeepReason.NOT_SURE to 1), actual = rebuilt.pages.single().keptReasons)
            assertFalse(actual = "Vermerk" in textOf(rebuilt = rebuilt))
            val original = Png.decode(image = (prepare() as StorageResult.Success).value.image)
            val kept = renderOf(rebuilt = rebuilt)
            assertEquals(
                expected = darkest(image = original),
                actual = darkest(image = kept),
                message = "the line keeps its grey",
            )
        }
    }

    @Test
    fun `a large shaded box is stored as JPEG and keeps its grey value`() {
        runTest {
            val scan = TestScans()
            scan.box(x = 200, y = 200, boxWidth = 900, boxHeight = 700, color = Color(SHADE, SHADE, SHADE))
            scan.writePdf(file = root.resolve("scan.pdf"))

            val rebuilt = rebuild(input = read(layout = PageLayoutModel(lines = emptyList(), graphics = emptyList())))

            val filters = imagesOf(rebuilt = rebuilt).map { it.cosObject.getCOSName(COSName.FILTER) }
            assertTrue(actual = COSName.DCT_DECODE in filters, message = "filters $filters")
            val centre = renderOf(rebuilt = rebuilt).getRGB(650, 550) and BYTE
            assertTrue(actual = abs(centre - SHADE) <= 1, message = "grey $centre")
        }
    }

    @Test
    fun `a logo stays an image next to the text`() {
        runTest {
            val scan = TestScans()
            val line = scan.text(text = "Beispiel GmbH Musterstraße 1", x = 500, y = 300)
            scan.box(x = 150, y = 200, boxWidth = 250, boxHeight = 150, color = Color(LOGO_RED, 30, 30))
            scan.writePdf(file = root.resolve("scan.pdf"))

            val rebuilt = rebuild(input = read(layout = layoutOf(lines = listOf(line))))

            assertEquals(expected = 1, actual = rebuilt.pages.single().linesAsText)
            assertTrue(actual = "Musterstraße" in textOf(rebuilt = rebuilt))
            val logo = renderOf(rebuilt = rebuilt).getRGB(275, 275)
            assertEquals(expected = LOGO_RED, actual = logo shr 16 and BYTE, message = "the logo keeps its colour")
        }
    }

    @Test
    fun `a page the AI could not read keeps the original page`() {
        runTest {
            val scan = TestScans()
            scan.text(text = "Unverändert", x = 150, y = 300)
            scan.writePdf(file = root.resolve("scan.pdf"))

            val rebuilt = rebuild(input = PageInput.Failed(pageIndex = 0))

            assertEquals(expected = mapOf(KeepReason.PAGE_FAILED to 1), actual = rebuilt.pages.single().keptReasons)
            assertTrue(actual = darkPixels(image = renderOf(rebuilt = rebuilt)) > 0)
        }
    }

    @Test
    fun `a crop of a line is a PNG around its box`() {
        runTest {
            val scan = TestScans()
            val line = scan.text(text = "Zeile zum erneuten Lesen", x = 150, y = 300)
            scan.writePdf(file = root.resolve("scan.pdf"))
            val prepared = (prepare() as StorageResult.Success).value

            val crop = rebuilder.crop(prepared = prepared, box = line.box)

            assertEquals(expected = "image/png", actual = crop.mimeType)
            assertNotNull(actual = Png.decode(image = crop))
        }
    }

    @Test
    fun `a discarded rebuild is gone, and the next start removes leftovers`() {
        runTest {
            TestScans().writePdf(file = root.resolve("scan.pdf"))
            val first = rebuild(input = PageInput.Failed(pageIndex = 0))
            val second = rebuild(input = PageInput.Failed(pageIndex = 0))

            rebuilder.discard(rebuilt = first)
            assertFalse(actual = work.resolve("${first.id}.pdf").exists())
            assertTrue(actual = work.resolve("${second.id}.pdf").exists())
            PdfBoxRebuilder(locations = TestPdfs.locations(root = root), workDirectory = work)

            assertFalse(actual = work.resolve("${second.id}.pdf").exists())
        }
    }

    private suspend fun prepare(): StorageResult<PreparedPageModel> {
        return rebuilder.preparePage(folder = folder, path = TestPdfs.path(raw = "scan.pdf"), page = 0)
    }

    private suspend fun read(layout: PageLayoutModel): PageInput {
        val prepared = (prepare() as StorageResult.Success).value
        return PageInput.Read(prepared = prepared, layout = layout)
    }

    private suspend fun rebuild(input: PageInput): RebuiltFileModel {
        val written = rebuilder.write(
            folder = folder,
            source = TestPdfs.path(raw = "scan.pdf"),
            sourceHash = ContentHashModel(value = "0".repeat(n = 64)),
            pages = listOf(input),
        )
        return assertIs<StorageResult.Success<RebuiltFileModel>>(value = written).value
    }

    private fun layoutOf(lines: List<LayoutLineModel>): PageLayoutModel {
        return PageLayoutModel(lines = lines, graphics = emptyList())
    }

    private fun textOf(rebuilt: RebuiltFileModel): String {
        return PdfFiles.load(file = work.resolve("${rebuilt.id}.pdf")).use { PDFTextStripper().getText(it) }
    }

    private fun renderOf(rebuilt: RebuiltFileModel): BufferedImage {
        return PdfFiles.load(file = work.resolve("${rebuilt.id}.pdf")).use {
            PDFRenderer(it).renderImageWithDPI(0, 300f, ImageType.RGB)
        }
    }

    private fun imagesOf(rebuilt: RebuiltFileModel): List<PDImageXObject> {
        return PdfFiles.load(file = work.resolve("${rebuilt.id}.pdf")).use { pdf ->
            val resources = pdf.getPage(0).resources
            resources.xObjectNames.map { resources.getXObject(it) }.filterIsInstance<PDImageXObject>()
        }
    }

    private fun darkest(image: BufferedImage): Int {
        return (0 until image.height).minOf { y -> (0 until image.width).minOf { x -> image.getRGB(x, y) and BYTE } }
    }

    private fun darkPixels(image: BufferedImage): Int {
        return (0 until image.height step 2).sumOf { y ->
            (0 until image.width step 2).count { x -> (image.getRGB(x, y) and BYTE) < DARK }
        }
    }

    private companion object {
        const val ANGLE_TOLERANCE = 0.1 + 1e-9
        const val GREY_PRINT = 165
        const val SHADE = 205
        const val LOGO_RED = 200
        const val BYTE = 0xFF
        const val DARK = 200
    }
}
