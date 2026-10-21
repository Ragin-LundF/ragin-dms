package dms.ui.components

import java.util.Locale

/** File sizes as people read them: 512 B, 3.4 KB, 12.0 MB. */
object FileSizes {

    private const val STEP = 1024.0
    private val UNITS = listOf("KB", "MB", "GB", "TB")

    fun format(bytes: Long): String {
        if (bytes < STEP) {
            return "$bytes B"
        }
        var value = bytes / STEP
        var unit = 0
        while (value >= STEP && unit < UNITS.lastIndex) {
            value /= STEP
            unit++
        }
        return "%.1f %s".format(locale = Locale.ROOT, value, UNITS[unit])
    }
}
