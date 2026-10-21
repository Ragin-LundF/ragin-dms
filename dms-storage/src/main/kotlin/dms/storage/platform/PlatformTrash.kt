package dms.storage.platform

import java.awt.Desktop
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path
import java.time.Clock

/**
 * The system trash (Finder, Explorer, desktop environment). Where the system has none, for example
 * some Linux desktops or network shares, the entry goes to `.dms-trash/<time>/` inside the library
 * folder instead. Nothing is ever deleted permanently.
 */
class PlatformTrash(private val clock: Clock = Clock.systemUTC()) : SystemTrash {

    override fun moveToTrash(libraryRoot: Path, entry: Path): Boolean {
        if (moveToSystemTrash(entry = entry)) {
            return true
        }
        val target = libraryRoot.resolve(FALLBACK_FOLDER)
            .resolve(clock.millis().toString())
            .resolve(libraryRoot.relativize(entry).toString())
        return try {
            Files.createDirectories(target.parent)
            Files.move(entry, target)
            true
        } catch (ignored: IOException) {
            false
        }
    }

    // Desktop's trash API takes a java.io.File; there is no java.nio equivalent.
    private fun moveToSystemTrash(entry: Path): Boolean {
        if (!Desktop.isDesktopSupported() || !Desktop.getDesktop().isSupported(Desktop.Action.MOVE_TO_TRASH)) {
            return false
        }
        return try {
            Desktop.getDesktop().moveToTrash(entry.toFile())
        } catch (ignored: UnsupportedOperationException) {
            false
        } catch (ignored: SecurityException) {
            false
        }
    }

    companion object {
        const val FALLBACK_FOLDER: String = ".dms-trash"
    }
}
