package dms.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.TextUnitType
import dms.ui.resources.Res
import dms.ui.resources.geist_bold
import dms.ui.resources.geist_medium
import dms.ui.resources.geist_regular
import dms.ui.resources.geist_semibold
import org.jetbrains.compose.resources.Font

/**
 * Material 3 type scale in Geist with bolder, slightly tighter headings. The styles that show numbers
 * (stat values, dates, sizes, counts) use tabular figures, so digits line up in columns. Geist (SIL Open
 * Font License, see `files/geist-OFL.txt`) is bundled as a resource, so the scale with it is [withGeist].
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

    /** [typography] with every style set in Geist. */
    @Composable
    fun withGeist(): Typography {
        val regular = Font(resource = Res.font.geist_regular, weight = FontWeight.Normal)
        val medium = Font(resource = Res.font.geist_medium, weight = FontWeight.Medium)
        val semiBold = Font(resource = Res.font.geist_semibold, weight = FontWeight.SemiBold)
        val bold = Font(resource = Res.font.geist_bold, weight = FontWeight.Bold)
        return remember(regular, medium, semiBold, bold) {
            inFamily(family = FontFamily(regular, medium, semiBold, bold))
        }
    }

    private fun inFamily(family: FontFamily): Typography {
        val scale = typography
        return scale.copy(
            displayLarge = scale.displayLarge.copy(fontFamily = family),
            displayMedium = scale.displayMedium.copy(fontFamily = family),
            displaySmall = scale.displaySmall.copy(fontFamily = family),
            headlineLarge = scale.headlineLarge.copy(fontFamily = family),
            headlineMedium = scale.headlineMedium.copy(fontFamily = family),
            headlineSmall = scale.headlineSmall.copy(fontFamily = family),
            titleLarge = scale.titleLarge.copy(fontFamily = family),
            titleMedium = scale.titleMedium.copy(fontFamily = family),
            titleSmall = scale.titleSmall.copy(fontFamily = family),
            bodyLarge = scale.bodyLarge.copy(fontFamily = family),
            bodyMedium = scale.bodyMedium.copy(fontFamily = family),
            bodySmall = scale.bodySmall.copy(fontFamily = family),
            labelLarge = scale.labelLarge.copy(fontFamily = family),
            labelMedium = scale.labelMedium.copy(fontFamily = family),
            labelSmall = scale.labelSmall.copy(fontFamily = family),
        )
    }
}
