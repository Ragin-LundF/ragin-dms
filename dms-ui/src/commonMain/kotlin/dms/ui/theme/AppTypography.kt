package dms.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType

/**
 * Material 3 type scale with bolder, slightly tighter headings. The styles that show numbers (stat values,
 * dates, sizes, counts) use tabular figures, so digits line up in columns.
 */
object AppTypography {

    private val tight: TextUnit = TextUnit(value = -0.4f, type = TextUnitType.Sp)
    private val base: Typography = Typography()
    private const val TABULAR_FIGURES = "tnum"

    val typography: Typography = base.copy(
        displaySmall = base.displaySmall.copy(fontWeight = FontWeight.Bold, letterSpacing = tight),
        headlineLarge = base.headlineLarge.copy(fontWeight = FontWeight.Bold, letterSpacing = tight),
        headlineMedium = base.headlineMedium.copy(
            fontWeight = FontWeight.Bold,
            letterSpacing = tight,
            fontFeatureSettings = TABULAR_FIGURES,
        ),
        headlineSmall = base.headlineSmall.copy(fontWeight = FontWeight.SemiBold, letterSpacing = tight),
        titleLarge = base.titleLarge.copy(fontWeight = FontWeight.SemiBold),
        titleMedium = base.titleMedium.copy(fontWeight = FontWeight.SemiBold),
        titleSmall = base.titleSmall.copy(fontWeight = FontWeight.SemiBold),
        labelLarge = base.labelLarge.copy(fontWeight = FontWeight.SemiBold),
        labelMedium = base.labelMedium.copy(fontWeight = FontWeight.Medium, fontFeatureSettings = TABULAR_FIGURES),
        bodySmall = base.bodySmall.copy(fontFeatureSettings = TABULAR_FIGURES),
    )
}
