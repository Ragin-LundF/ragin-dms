package dms.ai.read

import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.PageImageModel
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.ImageIO

/** A photo or scanned image, downscaled for a vision model or for a thumbnail. */
object ImageContent {

    fun read(file: Path): DocumentContent {
        val image = Files.newInputStream(file).use { ImageIO.read(it) }
            ?: return DocumentContent.NotReadable(reason = "The image format is not supported")
        return DocumentContent.PageImages(pages = listOf(VisionImages.of(image = image)))
    }

    /** The image at most [maxSide] pixels on its long side; `null` for a format Java cannot read. */
    fun thumbnail(file: Path, maxSide: Int): PageImageModel? {
        val image = Files.newInputStream(file).use { ImageIO.read(it) } ?: return null
        return VisionImages.of(image = image, maxSide = maxSide)
    }
}
