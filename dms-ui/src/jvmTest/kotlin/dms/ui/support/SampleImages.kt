package dms.ui.support

import java.awt.image.BufferedImage
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.ImageIO

/** Writes tiny synthetic PNG files, which the AI reader sends as page images. */
object SampleImages {

    fun write(file: Path): Path {
        Files.createDirectories(file.parent)
        val image = BufferedImage(8, 8, BufferedImage.TYPE_INT_RGB)
        ImageIO.write(image, "png", file.toFile())
        return file
    }
}
