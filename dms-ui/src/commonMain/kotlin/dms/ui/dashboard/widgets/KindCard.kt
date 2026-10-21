package dms.ui.dashboard.widgets

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.PieChart
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.chart.DonutChart
import dms.ui.components.chart.model.ChartSlice
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.components.surface.KindStyles
import dms.ui.dashboard.stats.model.KindCount
import dms.ui.theme.ChartColors

/** The share of PDFs, images, Office files and others. */
@Composable
fun KindCard(kinds: List<KindCount>, modifier: Modifier = Modifier) {
    AppCard(modifier = modifier) {
        CardTitle(title = "File types", icon = Icons.Rounded.PieChart, tint = ChartColors.slot(index = 1))
        DonutChart(
            slices = kinds.map { entry ->
                ChartSlice(
                    label = KindStyles.label(kind = entry.kind),
                    value = entry.count,
                    color = KindStyles.color(kind = entry.kind),
                )
            },
            centerValue = kinds.sumOf { it.count }.toString(),
            centerLabel = "files",
        )
    }
}
