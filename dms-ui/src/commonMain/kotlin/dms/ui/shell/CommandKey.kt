package dms.ui.shell

import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.staticCompositionLocalOf

/**
 * How shortcuts are written on this platform: "⌘" on macOS, "Ctrl+" elsewhere. Provided by the
 * platform, which knows the system.
 */
val LocalCommandKey: ProvidableCompositionLocal<String> = staticCompositionLocalOf { "Ctrl+" }
