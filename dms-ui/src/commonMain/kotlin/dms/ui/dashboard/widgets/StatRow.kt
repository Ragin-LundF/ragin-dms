package dms.ui.dashboard.widgets

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.Description
import androidx.compose.material.icons.rounded.NotificationsActive
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.TaskAlt
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.FileSizes
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing
import java.util.Locale
import kotlin.math.roundToInt

/** The four headline figures: documents, storage, how much is organized, and what waits for a decision. */
@Composable
fun StatRow(state: DashboardUiStateModel, onOpenUncategorized: () -> Unit) {
    val stats = state.stats
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier.fillMaxWidth().height(intrinsicSize = IntrinsicSize.Min),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        val card = Modifier.weight(weight = 1f).fillMaxHeight()
        StatCard(
            title = "Documents",
            value = "%,d".format(locale = Locale.getDefault(), stats.documentCount),
            caption = if (stats.addedRecently > 0) {
                "+${stats.addedRecently} in the last 30 days"
            } else {
                "None added lately"
            },
            icon = Icons.Rounded.Description,
            tint = colors.primary,
            modifier = card,
        )
        StatCard(
            title = "Storage",
            value = FileSizes.format(bytes = stats.totalBytes),
            caption = "Kept in each of ${state.folders.size} library folders",
            icon = Icons.Rounded.Storage,
            tint = colors.secondary,
            modifier = card,
        )
        StatCard(
            title = "Categorized",
            value = "${(stats.categorizedShare * 100).roundToInt()} %",
            caption = "${stats.uncategorizedCount} without a category",
            icon = Icons.Rounded.Category,
            tint = colors.tertiary,
            modifier = card,
            progress = stats.categorizedShare,
            onClick = onOpenUncategorized.takeIf { stats.uncategorizedCount > 0 },
        )
        val open = state.openTaskCount
        StatCard(
            title = "Open tasks",
            value = open.toString(),
            caption = if (open == 0) "All caught up" else "Waiting for your decision",
            icon = if (open == 0) Icons.Rounded.TaskAlt else Icons.Rounded.NotificationsActive,
            tint = if (open == 0) AppColors.statusGood else AppColors.statusWarning,
            modifier = card,
        )
    }
}
