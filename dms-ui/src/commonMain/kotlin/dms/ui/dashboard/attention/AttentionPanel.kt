package dms.ui.dashboard.attention

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CountPill
import dms.ui.dashboard.attention.model.AttentionListModel
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/**
 * "Needs your attention": the most urgent rows, how many there are in all, and a way to every task.
 * [busyReason] names the library operation that keeps the copies of a duplicates task waiting.
 */
@Composable
fun AttentionPanel(
    attention: AttentionListModel,
    busyReason: String?,
    thumbnails: ThumbnailLoader,
    onShowAllTasks: () -> Unit,
    modifier: Modifier = Modifier,
) {
    AppCard(modifier = modifier, contentPadding = 0.dp) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(horizontal = Spacing.large, vertical = Spacing.small),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            Text(
                text = "Needs your attention",
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                softWrap = false,
            )
            if (attention.total > 0) {
                CountPill(count = attention.total)
            }
            ActionButtons(
                buttons = attention.headerButtons +
                    ActionButtonModel(label = "All tasks", style = ButtonStyle.TEXT, onClick = onShowAllTasks),
                enabled = true,
                busyReason = null,
                modifier = Modifier.weight(weight = 1f).padding(start = Spacing.small),
            )
        }
        if (attention.rows.isEmpty()) {
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Text(
                text = "Nothing needs you right now. New, missing and changed files appear here after a scan.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(all = Spacing.large),
            )
        }
        attention.rows.forEach { row ->
            HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            AttentionRowView(row = row, busyReason = busyReason, thumbnails = thumbnails)
        }
    }
}
