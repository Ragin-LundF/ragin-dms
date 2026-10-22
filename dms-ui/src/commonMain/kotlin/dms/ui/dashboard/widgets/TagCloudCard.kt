package dms.ui.dashboard.widgets

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.Spacing

/** The tags as a cloud: the more documents a tag has, the larger and stronger its chip. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TagCloudCard(points: List<ChartPointModel>, onOpenTag: (String) -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    val type = MaterialTheme.typography
    val max = points.maxOfOrNull { it.value }?.takeIf { it > 0 } ?: 1
    AppCard(modifier = modifier) {
        CardTitle(title = "Tags", icon = Icons.Rounded.LocalOffer, tint = colors.secondary, subtitle = "Sized by use")
        if (points.isEmpty()) {
            Text(text = "No tags yet", style = type.bodyMedium, color = colors.onSurfaceVariant)
        }
        FlowRow(
            modifier = Modifier.verticalScroll(state = rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            points.forEach { point ->
                val weight = point.value.toFloat() / max
                val style = when {
                    weight > 0.66f -> type.titleMedium
                    weight > 0.33f -> type.bodyLarge
                    else -> type.labelLarge
                }
                Surface(
                    onClick = { onOpenTag(point.id) },
                    shape = CircleShape,
                    color = colors.secondary.copy(alpha = 0.08f + 0.2f * weight),
                    contentColor = colors.onSurface,
                ) {
                    Text(
                        text = "#${point.label}  ${point.value}",
                        style = style,
                        maxLines = 1,
                        softWrap = false,
                        modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.xsmall),
                    )
                }
            }
        }
    }
}
