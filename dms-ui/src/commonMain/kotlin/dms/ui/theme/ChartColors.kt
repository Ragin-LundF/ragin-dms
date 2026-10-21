package dms.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/**
 * Colours of chart series. The categorical slots are a validated colour-blind-safe order with a
 * light and a dark step per hue; use them in this order, never cycled. [neutral] is for "other".
 */
object ChartColors {

    private val lightSlots: List<Color> = listOf(
        Color(color = 0xFF2A78D6),
        Color(color = 0xFFEB6834),
        Color(color = 0xFF1BAF7A),
        Color(color = 0xFFEDA100),
    )

    private val darkSlots: List<Color> = listOf(
        Color(color = 0xFF3987E5),
        Color(color = 0xFFD95926),
        Color(color = 0xFF199E70),
        Color(color = 0xFFC98500),
    )

    val neutral: Color = Color(color = 0xFF898781)

    /** The categorical slot [index] (0-based) for the current light or dark theme. */
    @Composable
    fun slot(index: Int): Color {
        val slots = if (isSystemInDarkTheme()) darkSlots else lightSlots
        return slots[index]
    }
}
