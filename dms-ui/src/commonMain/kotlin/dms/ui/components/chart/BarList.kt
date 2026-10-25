package dms.ui.components.chart

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.theme.Spacing

/**
 * A ranked list: each entry with its count and a thin bar of its size in the brand colour, the same
 * in every list; a click opens the entry.
 */
@Composable
fun BarList(
    points: List<ChartPointModel>,
    onClick: (ChartPointModel) -> Unit,
    modifier: Modifier = Modifier,
) {
    val max = points.maxOfOrNull { it.value }?.takeIf { it > 0 } ?: 1
    val colors = MaterialTheme.colorScheme
    val bar = colors.primary.copy(alpha = 0.75f)
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
        points.forEach { point ->
            val grown = growIn(target = point.value.toFloat() / max)
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(shape = MaterialTheme.shapes.small)
                    .clickable { onClick(point) }
                    .padding(horizontal = Spacing.small, vertical = Spacing.xsmall),
            ) {
                Row(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = point.label,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(weight = 1f),
                    )
                    Text(
                        text = point.value.toString(),
                        style = MaterialTheme.typography.labelMedium,
                        color = colors.onSurfaceVariant,
                    )
                }
                Box(
                    modifier = Modifier
                        .padding(top = Spacing.xsmall)
                        .fillMaxWidth()
                        .height(height = 6.dp)
                        .background(color = colors.surfaceContainer, shape = CircleShape),
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth(fraction = grown.coerceAtLeast(minimumValue = 0.02f))
                            .height(height = 6.dp)
                            .background(color = bar, shape = CircleShape),
                    )
                }
            }
        }
    }
}
