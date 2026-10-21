package dms.storage.platform

import java.nio.file.Path
import java.util.Locale

/**
 * Where the tool keeps the state of this device: settings, journal, tasks, activity and scan caches.
 * Never inside a library folder.
 */
object AppDataDirectory {

    fun forApplication(name: String): Path {
        val os = System.getProperty("os.name").orEmpty().lowercase(Locale.ROOT)
        val home = Path.of(System.getProperty("user.home"))
        val base = when {
            os.contains(other = "mac") -> home.resolve("Library/Application Support")
            os.contains(other = "win") -> environmentPath(name = "APPDATA") ?: home.resolve("AppData/Roaming")
            else -> environmentPath(name = "XDG_DATA_HOME") ?: home.resolve(".local/share")
        }
        return base.resolve(name)
    }

    private fun environmentPath(name: String): Path? {
        return System.getenv(name)?.takeIf { it.isNotBlank() }?.let { Path.of(it) }
    }
}
