package dms.pdf.page

import dms.application.port.ai.model.PageImageModel
import dms.pdf.io.PdfFiles
import org.apache.pdfbox.rendering.ImageType
import org.apache.pdfbox.rendering.PDFRenderer
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import java.nio.file.Path
import javax.imageio.ImageIO
import kotlin.math.max
import kotlin.math.roundToInt

/** Pictures of single pages for the page editor and the rebuild comparison, as PNG. */
object PageImages {

    /** Enough for thumbnails; larger pictures are rendered at the resolution their size needs. */
    private const val MIN_DPI = 48f
    private const val MAX_DPI = 150f
    private const val POINTS_PER_INCH = 72f

    fun render(file: Path, page: Int, maxSide: Int): PageImageModel? {
        PdfFiles.load(file = file).use { pdf ->
            if (page !in 0 until pdf.numberOfPages) {
                return null
            }
            val box = pdf.getPage(page).cropBox
            val needed = maxSide * POINTS_PER_INCH / max(box.width, box.height)
            val dpi = needed.coerceIn(minimumValue = MIN_DPI, maximumValue = MAX_DPI)
            val image = PDFRenderer(pdf).renderImageWithDPI(page, dpi, ImageType.RGB)
            return encode(image = scaled(image = image, maxSide = maxSide))
        }
    }

    private fun scaled(image: BufferedImage, maxSide: Int): BufferedImage {
        val longest = max(image.width, image.height)
        if (longest <= maxSide) {
            return image
        }
        val factor = maxSide.toDouble() / longest
        val width = (image.width * factor).roundToInt().coerceAtLeast(minimumValue = 1)
        val height = (image.height * factor).roundToInt().coerceAtLeast(minimumValue = 1)
        val result = BufferedImage(width, height, BufferedImage.TYPE_INT_RGB)
        val graphics = result.createGraphics()
        graphics.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BILINEAR)
        graphics.drawImage(image, 0, 0, width, height, null)
        graphics.dispose()
        return result
    }

    private fun encode(image: BufferedImage): PageImageModel {
        val bytes = ByteArrayOutputStream().use { output ->
            ImageIO.write(image, "png", output)
            output.toByteArray()
        }
        return PageImageModel(bytes = bytes, mimeType = "image/png")
    }
}
