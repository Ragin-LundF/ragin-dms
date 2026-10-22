package dms.ui.components.chart

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.theme.Spacing

/**
 * Vertical bars of one series on a common baseline. The last bar and the hovered bar carry their
 * value; every bar is labelled below. With [onClick] a bar opens what it counts.
 */
@Composable
fun ColumnChart(
    points: List<ChartPointModel>,
    color: Color,
    modifier: Modifier = Modifier,
    height: Dp = 180.dp,
    onClick: ((ChartPointModel) -> Unit)? = null,
) {
    val max = points.maxOfOrNull { it.value }?.takeIf { it > 0 } ?: 1
    Column(modifier = modifier) {
        Row(
            modifier = Modifier.fillMaxWidth().height(height = height),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            verticalAlignment = Alignment.Bottom,
        ) {
            points.forEachIndexed { index, point ->
                ColumnBar(
                    point = point,
                    fraction = point.value.toFloat() / max,
                    color = color,
                    alwaysLabelled = index == points.lastIndex,
                    onClick = onClick?.let { open -> { open(point) } },
                    modifier = Modifier.weight(weight = 1f).fillMaxHeight(),
                )
            }
        }
        HorizontalDivider(color = MaterialTheme.colorScheme.outline)
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            points.forEach { point ->
                Text(
                    text = point.label,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    softWrap = false,
                    modifier = Modifier.weight(weight = 1f),
                    textAlign = TextAlign.Center,
                )
            }
        }
    }
}
