package dms.ai.read

import dms.application.port.ai.model.DocumentContent
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.ImageIO

/** A photo or scanned image, downscaled for a vision model. */
object ImageContent {

    fun read(file: Path): DocumentContent {
        val image = Files.newInputStream(file).use { ImageIO.read(it) }
            ?: return DocumentContent.NotReadable(reason = "The image format is not supported")
        return DocumentContent.PageImages(pages = listOf(VisionImages.of(image = image)))
    }
}
