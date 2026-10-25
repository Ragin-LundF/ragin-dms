package dms.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

/** The colour schemes of the app: indigo as the brand colour on neutral surfaces, teal and pink as accents. */
object AppColors {

    val light: ColorScheme = lightColorScheme(
        primary = Color(color = 0xFF4F46E5),
        onPrimary = Color.White,
        primaryContainer = Color(color = 0xFFEEEDFD),
        onPrimaryContainer = Color(color = 0xFF3730A3),
        secondary = Color(color = 0xFF0D9488),
        onSecondary = Color.White,
        secondaryContainer = Color(color = 0xFFCCFBF1),
        onSecondaryContainer = Color(color = 0xFF042F2E),
        tertiary = Color(color = 0xFFDB2777),
        onTertiary = Color.White,
        tertiaryContainer = Color(color = 0xFFFCE7F3),
        onTertiaryContainer = Color(color = 0xFF500724),
        background = Color(color = 0xFFF4F4F7),
        onBackground = Color(color = 0xFF17171E),
        surface = Color(color = 0xFFFFFFFF),
        onSurface = Color(color = 0xFF17171E),
        surfaceVariant = Color(color = 0xFFF1F1F5),
        onSurfaceVariant = Color(color = 0xFF696977),
        surfaceContainerLowest = Color(color = 0xFFFFFFFF),
        surfaceContainerLow = Color(color = 0xFFFBFBFC),
        surfaceContainer = Color(color = 0xFFF1F1F5),
        surfaceContainerHigh = Color(color = 0xFFE8E8EE),
        surfaceContainerHighest = Color(color = 0xFFE1E1E8),
        outline = Color(color = 0xFFC9C9D3),
        outlineVariant = Color(color = 0xFFE3E3EA),
        error = Color(color = 0xFFDC2626),
        onError = Color.White,
        errorContainer = Color(color = 0xFFFEE2E2),
        onErrorContainer = Color(color = 0xFF7F1D1D),
    )

    val dark: ColorScheme = darkColorScheme(
        primary = Color(color = 0xFF8B85FF),
        onPrimary = Color(color = 0xFF14123A),
        primaryContainer = Color(color = 0xFF25234A),
        onPrimaryContainer = Color(color = 0xFFC9C6FF),
        secondary = Color(color = 0xFF5EEAD4),
        onSecondary = Color(color = 0xFF042F2E),
        secondaryContainer = Color(color = 0xFF115E59),
        onSecondaryContainer = Color(color = 0xFFCCFBF1),
        tertiary = Color(color = 0xFFF9A8D4),
        onTertiary = Color(color = 0xFF500724),
        tertiaryContainer = Color(color = 0xFF9D174D),
        onTertiaryContainer = Color(color = 0xFFFCE7F3),
        background = Color(color = 0xFF0E0E11),
        onBackground = Color(color = 0xFFECECF2),
        surface = Color(color = 0xFF1A1A1F),
        onSurface = Color(color = 0xFFECECF2),
        surfaceVariant = Color(color = 0xFF212127),
        onSurfaceVariant = Color(color = 0xFFA0A0AE),
        surfaceContainerLowest = Color(color = 0xFF1A1A1F),
        surfaceContainerLow = Color(color = 0xFF141417),
        surfaceContainer = Color(color = 0xFF212127),
        surfaceContainerHigh = Color(color = 0xFF2A2A31),
        surfaceContainerHighest = Color(color = 0xFF33333B),
        outline = Color(color = 0xFF45454F),
        outlineVariant = Color(color = 0xFF2B2B33),
        error = Color(color = 0xFFF87171),
        onError = Color(color = 0xFF450A0A),
        errorContainer = Color(color = 0xFF7F1D1D),
        onErrorContainer = Color(color = 0xFFFEE2E2),
    )

    /** Status colours, never used for data series; always shown together with an icon or a label. */
    val statusGood: Color = Color(color = 0xFF0CA30C)
    val statusWarning: Color = Color(color = 0xFFFAB219)

    /**
     * The gradient of everything the AI does, and of the logo; nothing else uses it, so it reads as "AI".
     * White text stays readable on every stop.
     */
    /** Text and icons on [aiGradient]. */
    val onAiGradient: Color = Color.White

    val aiGradient: List<Color> = listOf(
        Color(color = 0xFF4338CA),
        Color(color = 0xFF7C3AED),
        Color(color = 0xFFDB2777),
    )
}
