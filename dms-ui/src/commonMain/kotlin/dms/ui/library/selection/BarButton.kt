package dms.ui.library.selection

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.vector.ImageVector
import dms.ui.components.ReasonTooltip
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * A button of the bulk bar, light on the bar's dark surface; [ai] gives it the AI gradient. While
 * [disabledReason] is set it is disabled and says why in its tooltip.
 */
@Composable
fun BarButton(
    label: String,
    icon: ImageVector,
    disabledReason: String?,
    ai: Boolean = false,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.medium
    val background = if (ai && disabledReason == null) {
        Modifier.clip(shape = shape).background(brush = Brush.linearGradient(colors = AppColors.aiGradient))
    } else {
        Modifier
    }
    ReasonTooltip(reason = disabledReason) {
        TextButton(
            onClick = onClick,
            enabled = disabledReason == null,
            shape = shape,
            modifier = background,
            colors = ButtonDefaults.textButtonColors(
                contentColor = if (ai) AppColors.onAiGradient else colors.inverseOnSurface,
                disabledContentColor = colors.inverseOnSurface.copy(alpha = DISABLED_ALPHA),
            ),
        ) {
            Icon(imageVector = icon, contentDescription = null, modifier = Modifier.size(size = Spacing.large))
            Spacer(modifier = Modifier.width(width = Spacing.xsmall + Spacing.hairline * 2))
            Text(text = label, maxLines = 1, softWrap = false)
        }
    }
}

private const val DISABLED_ALPHA = 0.5f
