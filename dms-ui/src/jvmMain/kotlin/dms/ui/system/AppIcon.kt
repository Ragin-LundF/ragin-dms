package dms.ui.system

import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.graphics.toComposeImageBitmap
import java.awt.Taskbar
import java.awt.image.BufferedImage
import javax.imageio.ImageIO

/** The logo from the classpath, for the window, the macOS dock and the brand mark. */
object AppIcon {

    private const val RESOURCE = "/app-icon.png"

    private val image: BufferedImage? by lazy {
        AppIcon::class.java.getResourceAsStream(RESOURCE)?.use { ImageIO.read(it) }
    }

    val painter: Painter? by lazy {
        image?.let { BitmapPainter(image = it.toComposeImageBitmap()) }
    }

    /** Shows the logo in the macOS dock, where the window icon is not used. */
    fun installInDock() {
        val icon = image ?: return
        if (Taskbar.isTaskbarSupported() && Taskbar.getTaskbar().isSupported(Taskbar.Feature.ICON_IMAGE)) {
            Taskbar.getTaskbar().iconImage = icon
        }
    }
}
