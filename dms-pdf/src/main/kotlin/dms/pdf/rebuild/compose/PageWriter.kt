package dms.pdf.rebuild.compose

import dms.pdf.rebuild.compose.model.CropModel
import dms.pdf.rebuild.fit.model.FittedLine
import dms.pdf.rebuild.font.FontSet
import org.apache.pdfbox.cos.COSName
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import org.apache.pdfbox.pdmodel.PDPageContentStream
import org.apache.pdfbox.pdmodel.common.PDRectangle
import org.apache.pdfbox.pdmodel.graphics.image.JPEGFactory
import org.apache.pdfbox.pdmodel.graphics.color.PDDeviceGray
import org.apache.pdfbox.pdmodel.graphics.image.LosslessFactory
import org.apache.pdfbox.pdmodel.graphics.image.PDImageXObject
import org.apache.pdfbox.util.Matrix
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.util.zip.DeflaterOutputStream

/** Writes one rebuilt page: a white page, the image crops at their exact place, then the real text on top. */
object PageWriter {

    const val DPI: Float = 300f
    private const val TO_POINTS = 72f / DPI
    private const val PHOTO_PIXELS = 300 * 300
    private const val JPEG_QUALITY = 0.85f
    private const val SHADED = 0.4
    private const val BITS_PER_SAMPLE = 8

    fun add(
        document: PDDocument,
        fonts: FontSet,
        width: Int,
        height: Int,
        crops: List<CropModel>,
        lines: List<FittedLine>,
    ) {
        val heightPt = height * TO_POINTS
        val page = PDPage(PDRectangle(width * TO_POINTS, heightPt))
        document.addPage(page)
        PDPageContentStream(document, page).use { content ->
            crops.forEach { crop -> draw(document = document, content = content, crop = crop, heightPt = heightPt) }
            content.beginText()
            lines.filterIsInstance<FittedLine.Text>().forEach { line ->
                write(content = content, fonts = fonts, line = line, heightPt = heightPt)
            }
            content.endText()
        }
    }

    private fun draw(document: PDDocument, content: PDPageContentStream, crop: CropModel, heightPt: Float) {
        // Colour pictures and large shaded areas compress far better as JPEG; line art and text stay lossless.
        val large = crop.image.width * crop.image.height > PHOTO_PIXELS &&
            (crop.image.type == BufferedImage.TYPE_INT_RGB || crop.filled > SHADED)
        val image = when {
            large -> JPEGFactory.createFromImage(document, crop.image, JPEG_QUALITY)
            crop.image.type == BufferedImage.TYPE_BYTE_GRAY -> losslessGray(document = document, image = crop.image)
            else -> LosslessFactory.createFromImage(document, crop.image)
        }
        content.drawImage(
            image,
            crop.box.x * TO_POINTS,
            heightPt - (crop.box.y + crop.box.height) * TO_POINTS,
            crop.box.width * TO_POINTS,
            crop.box.height * TO_POINTS,
        )
    }

    /**
     * The grey samples as they are. LosslessFactory reads grey images through getRGB, which treats them
     * as linear grey and lightens every mid-tone.
     */
    private fun losslessGray(document: PDDocument, image: BufferedImage): PDImageXObject {
        val samples = ByteArray(size = image.width * image.height)
        image.raster.getDataElements(0, 0, image.width, image.height, samples)
        val deflated = ByteArrayOutputStream().use { output ->
            DeflaterOutputStream(output).use { it.write(samples) }
            output.toByteArray()
        }
        return PDImageXObject(
            document,
            ByteArrayInputStream(deflated),
            COSName.FLATE_DECODE,
            image.width,
            image.height,
            BITS_PER_SAMPLE,
            PDDeviceGray.INSTANCE,
        )
    }

    private fun write(content: PDPageContentStream, fonts: FontSet, line: FittedLine.Text, heightPt: Float) {
        content.setFont(fonts.of(style = line.style), line.sizePx * TO_POINTS)
        content.setNonStrokingColor(line.color)
        val y = heightPt - line.baseline * TO_POINTS
        line.runs.forEachIndexed { i, run ->
            content.setHorizontalScaling(run.scalePercent)
            content.setTextMatrix(Matrix.getTranslateInstance(run.ink.x * TO_POINTS, y))
            // A trailing space keeps words apart when the text is copied.
            content.showText(if (i < line.runs.size - 1) run.text + " " else run.text)
        }
    }
}
