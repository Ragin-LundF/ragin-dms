package dms.ai.read

import dms.application.port.ai.model.PageImageModel
import java.awt.Color
import java.awt.RenderingHints
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.IIOImage
import javax.imageio.ImageIO
import javax.imageio.ImageWriteParam
import kotlin.math.max
import kotlin.math.roundToInt

/**
 * Prepares an image for a vision model or the screen: at most [MAX_SIDE] pixels on the long side, as JPEG,
 * or for reading a layout, as lossless PNG on the patch grid of Qwen-VL models.
 */
object VisionImages {

    const val MAX_SIDE: Int = 1600
    private const val QUALITY = 0.85f

    /** The long side of a page sent for its layout. */
    const val LAYOUT_SIDE: Int = 1568
    private const val PATCH = 28

    fun of(image: BufferedImage, maxSide: Int = MAX_SIDE): PageImageModel {
        val scale = minOf(1.0, maxSide.toDouble() / max(image.width, image.height))
        val width = max(1, (image.width * scale).roundToInt())
        val height = max(1, (image.height * scale).roundToInt())
        // JPEG has no alpha channel, so transparent parts become white.
        val target = BufferedImage(width, height, BufferedImage.TYPE_INT_RGB)
        val graphics = target.createGraphics()
        try {
            graphics.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BILINEAR)
            graphics.color = Color.WHITE
            graphics.fillRect(0, 0, width, height)
            graphics.drawImage(image, 0, 0, width, height, null)
        } finally {
            graphics.dispose()
        }
        return PageImageModel(bytes = jpegOf(image = target), mimeType = "image/jpeg")
    }

    /** At most [LAYOUT_SIDE] on the long side, both sides a multiple of 28 (the patch size of Qwen-VL models). */
    fun layoutPng(image: BufferedImage): BufferedImage {
        val scale = minOf(1.0, LAYOUT_SIDE.toDouble() / max(image.width, image.height))
        val width = max(PATCH, (image.width * scale / PATCH).roundToInt() * PATCH)
        val height = max(PATCH, (image.height * scale / PATCH).roundToInt() * PATCH)
        val target = BufferedImage(width, height, BufferedImage.TYPE_INT_RGB)
        val graphics = target.createGraphics()
        try {
            graphics.setRenderingHint(RenderingHints.KEY_INTERPOLATION, RenderingHints.VALUE_INTERPOLATION_BICUBIC)
            graphics.drawImage(image, 0, 0, width, height, null)
        } finally {
            graphics.dispose()
        }
        return target
    }

    fun pngOf(image: BufferedImage): PageImageModel {
        val output = ByteArrayOutputStream()
        ImageIO.write(image, "png", output)
        return PageImageModel(bytes = output.toByteArray(), mimeType = "image/png")
    }

    private fun jpegOf(image: BufferedImage): ByteArray {
        val writer = ImageIO.getImageWritersByFormatName("jpeg").next()
        val output = ByteArrayOutputStream()
        ImageIO.createImageOutputStream(output).use { stream ->
            writer.output = stream
            val parameters = writer.defaultWriteParam.apply {
                compressionMode = ImageWriteParam.MODE_EXPLICIT
                compressionQuality = QUALITY
            }
            writer.write(null, IIOImage(image, null, null), parameters)
            writer.dispose()
        }
        return output.toByteArray()
    }
}
