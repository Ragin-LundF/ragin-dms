package dms.pdf.rebuild

import dms.application.port.ai.model.PageImageModel
import java.awt.image.BufferedImage
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.IOException
import javax.imageio.ImageIO

/** Page images cross the port as PNG: lossless, so the masks see the pixels the AI saw. */
object Png {

    private const val MIME_TYPE = "image/png"

    fun encode(image: BufferedImage): PageImageModel {
        val bytes = ByteArrayOutputStream().use { output ->
            ImageIO.write(image, "png", output)
            output.toByteArray()
        }
        return PageImageModel(bytes = bytes, mimeType = MIME_TYPE)
    }

    fun decode(image: PageImageModel): BufferedImage {
        return ImageIO.read(ByteArrayInputStream(image.bytes)) ?: throw IOException("Not a readable page image")
    }
}
