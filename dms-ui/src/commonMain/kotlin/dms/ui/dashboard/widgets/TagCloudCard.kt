package dms.ui.dashboard.widgets

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
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

/**
 * The tags as light chips in the order of [points], most used first, each with its count; a click
 * opens the tag in the library.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TagCloudCard(points: List<ChartPointModel>, onOpenTag: (String) -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    AppCard(modifier = modifier) {
        CardTitle(
            title = "Tags",
            icon = Icons.Rounded.LocalOffer,
            tint = colors.secondary,
            subtitle = "Most used first",
        )
        if (points.isEmpty()) {
            Text(text = "No tags yet", style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant)
        }
        FlowRow(
            modifier = Modifier.verticalScroll(state = rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            points.forEach { point ->
                TagChip(point = point, onClick = { onOpenTag(point.id) })
            }
        }
    }
}

@Composable
private fun TagChip(point: ChartPointModel, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Surface(
        onClick = onClick,
        shape = CircleShape,
        color = colors.surfaceContainer,
        contentColor = colors.onSurface,
        border = BorderStroke(width = Spacing.hairline, color = colors.outlineVariant),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.xsmall),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            Text(text = point.label, style = MaterialTheme.typography.labelLarge, maxLines = 1, softWrap = false)
            Text(
                text = point.value.toString(),
                style = MaterialTheme.typography.labelMedium,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                softWrap = false,
            )
        }
    }
}
