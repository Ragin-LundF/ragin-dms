package dms.ui.components.layout

import androidx.compose.runtime.ProvidableCompositionLocal
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.input.pointer.PointerIcon

/**
 * The pointer shown over a [ResizeHandle], provided by the platform, which knows its cursors. The default
 * pointer where none is provided, such as in tests.
 */
val LocalResizePointerIcon: ProvidableCompositionLocal<PointerIcon> = staticCompositionLocalOf { PointerIcon.Default }
