package dms.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Shapes
import androidx.compose.ui.unit.Dp

/** Generous rounding: chips and fields small, cards large, dialogs extra large. */
object AppShapes {

    val shapes: Shapes = Shapes(
        extraSmall = RoundedCornerShape(size = Dp(value = 6f)),
        small = RoundedCornerShape(size = Dp(value = 10f)),
        medium = RoundedCornerShape(size = Dp(value = 14f)),
        large = RoundedCornerShape(size = Dp(value = 20f)),
        extraLarge = RoundedCornerShape(size = Dp(value = 28f)),
    )
}
