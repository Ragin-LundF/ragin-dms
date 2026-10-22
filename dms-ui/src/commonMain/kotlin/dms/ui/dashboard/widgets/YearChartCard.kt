package dms.ui.dashboard.widgets

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.BarChart
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.chart.ColumnChart
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.ChartColors

/** How many documents each year holds; a click on a year opens its documents in the library. */
@Composable
fun YearChartCard(points: List<ChartPointModel>, onOpenYear: (String) -> Unit, modifier: Modifier = Modifier) {
    AppCard(modifier = modifier) {
        CardTitle(
            title = "Documents by year",
            icon = Icons.Rounded.BarChart,
            tint = ChartColors.slot(index = 0),
            subtitle = "By document date, or by when they were added. Click a year to open it",
        )
        if (points.isEmpty()) {
            Text(
                text = "No documents yet",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            ColumnChart(points = points, color = ChartColors.slot(index = 0), onClick = { onOpenYear(it.id) })
        }
    }
}
