package dms.ui.components.chart

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.model.ChartSliceModel
import dms.ui.theme.Spacing
import kotlin.math.roundToInt

/** A donut with the total in its centre and a legend that names every slice with its count and share. */
@Composable
fun DonutChart(
    slices: List<ChartSliceModel>,
    centerValue: String,
    centerLabel: String,
    modifier: Modifier = Modifier,
    diameter: Dp = 148.dp,
) {
    val total = slices.sumOf { it.value }.takeIf { it > 0 } ?: 1
    val progress = growIn(target = 1f)
    val track = MaterialTheme.colorScheme.surfaceContainerHigh
    Row(
        modifier = modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        Box(modifier = Modifier.size(size = diameter), contentAlignment = Alignment.Center) {
            Canvas(modifier = Modifier.size(size = diameter)) {
                drawDonut(
                    slices = slices,
                    total = total,
                    progress = progress,
                    track = track,
                    strokeWidth = 18.dp,
                    startAngle = -90f,
                    gapAngle = 2f,
                    fullCircle = 360f,
                )
            }
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text(text = centerValue, style = MaterialTheme.typography.headlineSmall)
                Text(
                    text = centerLabel,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            slices.forEach { slice ->
                LegendRow(slice = slice, share = (100f * slice.value / total).roundToInt())
            }
        }
    }
}
