package dms.ui.dashboard.header

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.format.FileSizes
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.panel.ActivityTexts
import dms.ui.theme.Spacing
import java.time.LocalTime

private const val NOON = 12
private const val EVENING = 18

/** One line on top of the dashboard: a greeting by the time of day and the size of the library. */
@Composable
fun DashboardHeader(state: DashboardUiStateModel) {
    val greeting = remember { greetingAt(hour = LocalTime.now().hour) }
    val scanned = state.lastScanAt?.let { "last scan ${ActivityTexts.time(instant = it)}" } ?: "not scanned yet"
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Text(
            text = greeting,
            style = MaterialTheme.typography.headlineSmall,
            maxLines = 1,
            softWrap = false,
            modifier = Modifier.alignByBaseline(),
        )
        Text(
            text = "${state.stats.documentCount} documents · ${FileSizes.format(bytes = state.stats.totalBytes)}" +
                " · ${state.folders.size} library folders · $scanned",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.alignByBaseline(),
        )
    }
}

private fun greetingAt(hour: Int): String {
    return when {
        hour < NOON -> "Good morning"
        hour < EVENING -> "Good afternoon"
        else -> "Good evening"
    }
}
