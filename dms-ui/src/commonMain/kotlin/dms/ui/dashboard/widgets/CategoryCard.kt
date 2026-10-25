package dms.ui.dashboard.widgets

import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.chart.BarList
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle

/** The largest categories and the uncategorized documents; a click opens them in the library. */
@Composable
fun CategoryCard(
    points: List<ChartPointModel>,
    uncategorizedCount: Int,
    onOpenCategory: (String) -> Unit,
    onOpenUncategorized: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    AppCard(modifier = modifier) {
        CardTitle(
            title = "Categories",
            icon = Icons.Rounded.Category,
            tint = colors.tertiary,
            subtitle = "Click one to open its documents",
        )
        val uncategorized = ChartPointModel(id = "", label = "Uncategorized", value = uncategorizedCount)
        val all = points + listOfNotNull(uncategorized.takeIf { uncategorizedCount > 0 })
        if (all.isEmpty()) {
            Text(
                text = "No documents yet",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
            )
        }
        BarList(
            points = all,
            onClick = { point -> if (point.id.isEmpty()) onOpenUncategorized() else onOpenCategory(point.id) },
            modifier = Modifier.verticalScroll(state = rememberScrollState()),
        )
    }
}
