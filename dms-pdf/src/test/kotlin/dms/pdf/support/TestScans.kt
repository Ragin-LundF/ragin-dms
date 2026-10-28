package dms.pdf.support

import dms.domain.rebuild.FontFamily
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.LayoutLineModel
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import org.apache.pdfbox.pdmodel.PDPageContentStream
import org.apache.pdfbox.pdmodel.common.PDRectangle
import org.apache.pdfbox.pdmodel.graphics.image.LosslessFactory
import java.awt.Color
import java.awt.Font
import java.awt.RenderingHints
import java.awt.geom.AffineTransform
import java.awt.image.BufferedImage
import java.nio.file.Files
import java.nio.file.Path

/**
 * Synthetic scans: a page image at 300 dpi with text drawn in the bundled Liberation Sans, and the PDF
 * that holds it as one picture, like a scanner writes it. Boxes of the drawn text come back in 0..1000,
 * as the AI would answer them.
 */
class TestScans(val width: Int = WIDTH, val height: Int = HEIGHT) {

    val image: BufferedImage = BufferedImage(width, height, BufferedImage.TYPE_INT_RGB)
    private val graphics = image.createGraphics().apply {
        color = Color.WHITE
        fillRect(0, 0, width, height)
        setRenderingHint(RenderingHints.KEY_TEXT_ANTIALIASING, RenderingHints.VALUE_TEXT_ANTIALIAS_ON)
    }

    /** Draws [text] with its baseline at [x], [y] and returns the line as the AI would read it. */
    fun text(
        text: String,
        x: Int,
        y: Int,
        gray: Int = 0,
        legible: Boolean = true,
        read: String = text,
    ): LayoutLineModel {
        graphics.font = FONT
        graphics.color = Color(gray, gray, gray)
        graphics.drawString(text, x, y)
        val context = graphics.fontRenderContext
        val bounds = FONT.createGlyphVector(context, text).getPixelBounds(context, x.toFloat(), y.toFloat())
        return LayoutLineModel(
            box = LayoutBoxModel(
                left = bounds.x * LayoutBoxModel.SCALE / width,
                top = bounds.y * LayoutBoxModel.SCALE / height,
                right = (bounds.x + bounds.width) * LayoutBoxModel.SCALE / width,
                bottom = (bounds.y + bounds.height) * LayoutBoxModel.SCALE / height,
            ),
            text = read,
            bold = false,
            family = FontFamily.SANS,
            legible = legible,
        )
    }

    fun box(x: Int, y: Int, boxWidth: Int, boxHeight: Int, color: Color) {
        graphics.color = color
        graphics.fillRect(x, y, boxWidth, boxHeight)
    }

    /** The page turned by [degrees], clockwise, on white. */
    fun turned(degrees: Double): BufferedImage {
        val turned = BufferedImage(width, height, BufferedImage.TYPE_INT_RGB)
        val target = turned.createGraphics()
        target.color = Color.WHITE
        target.fillRect(0, 0, width, height)
        target.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BICUBIC)
        val turn = AffineTransform.getRotateInstance(Math.toRadians(degrees), width / 2.0, height / 2.0)
        target.drawImage(image, turn, null)
        target.dispose()
        return turned
    }

    /** A PDF with [page] as its only content, at 300 dpi. */
    fun writePdf(file: Path, page: BufferedImage = image) {
        Files.createDirectories(file.parent)
        PDDocument().use { pdf ->
            val pdfPage = PDPage(PDRectangle(page.width * TO_POINTS, page.height * TO_POINTS))
            pdf.addPage(pdfPage)
            PDPageContentStream(pdf, pdfPage).use { stream ->
                val picture = LosslessFactory.createFromImage(pdf, page)
                stream.drawImage(picture, 0f, 0f, pdfPage.mediaBox.width, pdfPage.mediaBox.height)
            }
            pdf.save(file.toFile())
        }
    }

    companion object {
        const val WIDTH: Int = 1600
        const val HEIGHT: Int = 1200
        private const val TO_POINTS = 72f / 300f

        /** 12 pt at 300 dpi. */
        val FONT: Font = TestScans::class.java.getResourceAsStream("/fonts/LiberationSans-Regular.ttf")!!.use {
            Font.createFont(Font.TRUETYPE_FONT, it).deriveFont(50f)
        }
    }
}
