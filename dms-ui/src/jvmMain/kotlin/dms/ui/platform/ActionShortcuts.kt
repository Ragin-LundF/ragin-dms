package dms.ui.platform

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyShortcut
import dms.ui.shell.palette.model.PaletteAction

/** The menu bar shortcut of a palette action, from the one list in [PaletteAction]; `null` without one. */
object ActionShortcuts {

    fun of(action: PaletteAction): KeyShortcut? {
        val key = keyOf(shortcutKey = action.shortcutKey ?: return null) ?: return null
        return HostOs.shortcut(key = key, shift = action.shift)
    }

    private fun keyOf(shortcutKey: String): Key? {
        return when (shortcutKey) {
            "R" -> Key.R
            "S" -> Key.S
            "M" -> Key.M
            "1" -> Key.One
            "2" -> Key.Two
            "3" -> Key.Three
            "," -> Key.Comma
            else -> null
        }
    }
}
