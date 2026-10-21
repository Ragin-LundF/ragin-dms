package dms.ui.theme

import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.painter.Painter

/**
 * The app's logo, provided by the platform that loads it. `null` where none is provided, such as in
 * tests; the brand mark then draws a plain tile.
 */
val LocalAppLogo: ProvidableCompositionLocal<Painter?> = staticCompositionLocalOf { null }
