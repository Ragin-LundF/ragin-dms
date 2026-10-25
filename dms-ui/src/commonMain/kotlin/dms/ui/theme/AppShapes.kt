package dms.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.ui.unit.Dp

/** Calm rounding: chips and fields small, cards large, dialogs extra large. */
object AppShapes {

    val shapes: Shapes = Shapes(
        extraSmall = RoundedCornerShape(size = Dp(value = 4f)),
        small = RoundedCornerShape(size = Dp(value = 6f)),
        medium = RoundedCornerShape(size = Dp(value = 8f)),
        large = RoundedCornerShape(size = Dp(value = 12f)),
        extraLarge = RoundedCornerShape(size = Dp(value = 16f)),
    )
}
