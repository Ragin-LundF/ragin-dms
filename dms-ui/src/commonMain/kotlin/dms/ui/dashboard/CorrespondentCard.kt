package dms.ui.dashboard

import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Business
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.chart.BarList
import dms.ui.components.chart.model.ChartPointModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle

/** Who the documents come from; a click finds that correspondent's documents. */
@Composable
fun CorrespondentCard(points: List<ChartPointModel>, onSearch: (String) -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    AppCard(modifier = modifier) {
        CardTitle(title = "Correspondents", icon = Icons.Rounded.Business, tint = colors.primary)
        if (points.isEmpty()) {
            Text(
                text = "Add a correspondent in a document's details to see it here",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
            )
        }
        BarList(
            points = points,
            color = colors.primary,
            onClick = { onSearch(it.label) },
            modifier = Modifier.verticalScroll(state = rememberScrollState()),
        )
    }
}
