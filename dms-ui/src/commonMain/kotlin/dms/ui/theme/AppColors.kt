package dms.ui.theme

import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color

/** The colour schemes of the app: indigo as the brand colour, teal and pink as accents, cool neutral surfaces. */
object AppColors {

    val light: ColorScheme = lightColorScheme(
        primary = Color(color = 0xFF4F46E5),
        onPrimary = Color.White,
        primaryContainer = Color(color = 0xFFE0E7FF),
        onPrimaryContainer = Color(color = 0xFF1E1B4B),
        secondary = Color(color = 0xFF0D9488),
        onSecondary = Color.White,
        secondaryContainer = Color(color = 0xFFCCFBF1),
        onSecondaryContainer = Color(color = 0xFF042F2E),
        tertiary = Color(color = 0xFFDB2777),
        onTertiary = Color.White,
        tertiaryContainer = Color(color = 0xFFFCE7F3),
        onTertiaryContainer = Color(color = 0xFF500724),
        background = Color(color = 0xFFF5F6FA),
        onBackground = Color(color = 0xFF0F172A),
        surface = Color(color = 0xFFFFFFFF),
        onSurface = Color(color = 0xFF0F172A),
        surfaceVariant = Color(color = 0xFFEEF0F6),
        onSurfaceVariant = Color(color = 0xFF5B6475),
        surfaceContainerLowest = Color(color = 0xFFFFFFFF),
        surfaceContainerLow = Color(color = 0xFFF9FAFC),
        surfaceContainer = Color(color = 0xFFF1F3F9),
        surfaceContainerHigh = Color(color = 0xFFE9ECF4),
        surfaceContainerHighest = Color(color = 0xFFE2E6F0),
        outline = Color(color = 0xFFC7CEDD),
        outlineVariant = Color(color = 0xFFE4E7EF),
        error = Color(color = 0xFFDC2626),
        onError = Color.White,
        errorContainer = Color(color = 0xFFFEE2E2),
        onErrorContainer = Color(color = 0xFF7F1D1D),
    )

    val dark: ColorScheme = darkColorScheme(
        primary = Color(color = 0xFFA5B4FC),
        onPrimary = Color(color = 0xFF1E1B4B),
        primaryContainer = Color(color = 0xFF3730A3),
        onPrimaryContainer = Color(color = 0xFFE0E7FF),
        secondary = Color(color = 0xFF5EEAD4),
        onSecondary = Color(color = 0xFF042F2E),
        secondaryContainer = Color(color = 0xFF115E59),
        onSecondaryContainer = Color(color = 0xFFCCFBF1),
        tertiary = Color(color = 0xFFF9A8D4),
        onTertiary = Color(color = 0xFF500724),
        tertiaryContainer = Color(color = 0xFF9D174D),
        onTertiaryContainer = Color(color = 0xFFFCE7F3),
        background = Color(color = 0xFF0B0E17),
        onBackground = Color(color = 0xFFE6E9F2),
        surface = Color(color = 0xFF121624),
        onSurface = Color(color = 0xFFE6E9F2),
        surfaceVariant = Color(color = 0xFF1C2133),
        onSurfaceVariant = Color(color = 0xFF9AA3B8),
        surfaceContainerLowest = Color(color = 0xFF0B0E17),
        surfaceContainerLow = Color(color = 0xFF111522),
        surfaceContainer = Color(color = 0xFF171C2B),
        surfaceContainerHigh = Color(color = 0xFF1E2435),
        surfaceContainerHighest = Color(color = 0xFF262D40),
        outline = Color(color = 0xFF3A4258),
        outlineVariant = Color(color = 0xFF242A3B),
        error = Color(color = 0xFFF87171),
        onError = Color(color = 0xFF450A0A),
        errorContainer = Color(color = 0xFF7F1D1D),
        onErrorContainer = Color(color = 0xFFFEE2E2),
    )

    /** Status colours, never used for data series; always shown together with an icon or a label. */
    val statusGood: Color = Color(color = 0xFF0CA30C)
    val statusWarning: Color = Color(color = 0xFFFAB219)

    /** The brand gradient of the dashboard header and the logo; white text stays readable on every stop. */
    val brandGradient: List<Color> = listOf(
        Color(color = 0xFF4338CA),
        Color(color = 0xFF6D28D9),
        Color(color = 0xFFBE185D),
    )
}
