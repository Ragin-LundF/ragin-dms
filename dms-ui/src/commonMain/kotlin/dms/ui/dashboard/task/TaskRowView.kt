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
import dms.ui.dashboard.duplicate.DuplicateCopies
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskRowStateModel
import dms.ui.preview.FileThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/**
 * One open task with a thumbnail of its file (or the icon of its kind), its description, its actions
 * and buttons to open the file.
 */
@Composable
fun TaskRowView(row: TaskRowStateModel, group: TaskGroup, busyReason: String?, thumbnails: ThumbnailLoader) {
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
            val preview = row.preview
            if (preview == null) {
                IconBadge(icon = group.icon, tint = colors.primary, size = 36.dp)
            } else {
                FileThumbnail(
                    folderId = preview.folderId,
                    path = preview.path,
                    loader = thumbnails,
                    onClick = { (row.opens.firstOrNull() ?: row.copies.firstOrNull()?.open)?.onClick?.invoke() },
                ) {
                    IconBadge(icon = group.icon, tint = colors.primary, size = 36.dp)
                }
            }
            Column(modifier = Modifier.weight(weight = 1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = row.title,
                        style = MaterialTheme.typography.titleSmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(weight = 1f),
                    )
                    // Looking at a file never changes the library, so it stays possible while operations run.
                    ActionButtons(buttons = row.opens, enabled = true, busyReason = null)
                }
                if (row.detail.isNotEmpty()) {
                    Text(
                        text = row.detail,
                        style = MaterialTheme.typography.bodySmall,
                        color = colors.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                DuplicateCopies(
                    copies = row.copies,
                    busyReason = busyReason,
                    modifier = Modifier.padding(top = Spacing.small),
                )
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
