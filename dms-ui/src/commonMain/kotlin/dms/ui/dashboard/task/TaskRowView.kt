package dms.ui.dashboard.task

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.surface.IconBadge
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskRowState
import dms.ui.theme.Spacing

/** One open task with the icon of its kind, its description and its actions. */
@Composable
fun TaskRowView(row: TaskRowState, group: TaskGroup, busyReason: String?) {
    val colors = MaterialTheme.colorScheme
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.medium,
        color = colors.surfaceContainerLow,
    ) {
        Row(
            modifier = Modifier.padding(all = Spacing.medium),
            verticalAlignment = Alignment.Top,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            IconBadge(icon = group.icon, tint = colors.primary, size = 36.dp)
            Column(modifier = Modifier.weight(weight = 1f)) {
                Text(
                    text = row.title,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (row.detail.isNotEmpty()) {
                    Text(
                        text = row.detail,
                        style = MaterialTheme.typography.bodySmall,
                        color = colors.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                ActionButtons(
                    buttons = row.buttons,
                    enabled = busyReason == null,
                    busyReason = busyReason,
                    modifier = Modifier.padding(top = Spacing.small),
                )
            }
        }
    }
}
