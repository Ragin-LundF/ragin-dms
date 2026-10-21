package dms.ui.platform

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyShortcut
import java.util.Locale

/** The operating system this app runs on, and its conventions. */
object HostOs {

    private val name = System.getProperty("os.name").orEmpty().lowercase(Locale.ROOT)

    val isMac: Boolean = name.contains(other = "mac")

    val isWindows: Boolean = name.contains(other = "win")

    /** [key] with the platform's command modifier: ⌘ on macOS, Ctrl elsewhere. */
    fun shortcut(key: Key, shift: Boolean = false): KeyShortcut {
        return KeyShortcut(key = key, meta = isMac, ctrl = !isMac, shift = shift)
    }
}
