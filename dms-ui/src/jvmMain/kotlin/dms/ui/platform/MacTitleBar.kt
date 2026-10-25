package dms.ui.platform

import androidx.compose.ui.awt.ComposeWindow
import androidx.compose.ui.unit.Dp

/**
 * On macOS the content runs up under a transparent title bar, so the window's traffic lights sit inside
 * the navigation, which keeps [inset] free for them. Windows and Linux keep their usual title bar.
 */
object MacTitleBar {

    /** The room the traffic lights take at the top of the navigation; none on other systems. */
    val inset: Dp = if (HostOs.isMac) Dp(value = 28f) else Dp(value = 0f)

    fun applyTo(window: ComposeWindow) {
        if (!HostOs.isMac) {
            return
        }
        window.rootPane.putClientProperty("apple.awt.fullWindowContent", true)
        window.rootPane.putClientProperty("apple.awt.transparentTitleBar", true)
        window.rootPane.putClientProperty("apple.awt.windowTitleVisible", false)
    }
}
