package dms.ui.shell

import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.unit.Dp

/**
 * The room the window's own controls take at the top of the content, such as the macOS traffic lights
 * when the title bar is transparent. Provided by the platform; none by default.
 */
val LocalTitleBarInset: ProvidableCompositionLocal<Dp> = staticCompositionLocalOf { Dp(value = 0f) }
