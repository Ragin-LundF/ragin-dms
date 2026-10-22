package dms.ui.dashboard.task

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.TaskAlt
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.FullViewButton
import dms.ui.components.SectionHeader
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.components.surface.EmptyState
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskGroupState
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/**
 * The open tasks, grouped by kind, with a filter per kind. On the dashboard it offers [onFullView] to
 * show every task; on the tasks screen, where [onFullView] is `null`, it fills the screen.
 */
@Composable
fun TaskList(
    groups: List<TaskGroupState>,
    filter: TaskGroup?,
    busyReason: String?,
    thumbnails: ThumbnailLoader,
    onFilter: (TaskGroup?) -> Unit,
    onFullView: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    val total = groups.sumOf { it.rows.size }
    AppCard(modifier = modifier) {
        CardTitle(
            title = "Open tasks",
            icon = Icons.Rounded.TaskAlt,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = if (total == 0) "Nothing waits for you" else "$total waiting for your decision",
        ) {
            onFullView?.let { FullViewButton(label = "Show all tasks", onClick = it) }
        }
        if (total == 0) {
            EmptyState(
                icon = Icons.Rounded.TaskAlt,
                title = "All caught up",
                message = "New, missing and changed files appear here after a scan.",
                modifier = Modifier.fillMaxSize(),
            )
            return@AppCard
        }
        TaskFilterChips(groups = groups, filter = filter, total = total, onFilter = onFilter)
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            groups.filter { filter == null || it.group == filter }.forEach { group ->
                item(key = "group:${group.group.name}") {
                    SectionHeader(title = group.group.label) {
                        if (group.bulkActions.isNotEmpty()) {
                            ActionButtons(
                                buttons = group.bulkActions,
                                enabled = busyReason == null,
                                busyReason = busyReason,
                            )
                        }
                    }
                }
                items(items = group.rows, key = { it.key.value }) { row ->
                    TaskRowView(row = row, group = group.group, busyReason = busyReason, thumbnails = thumbnails)
                }
            }
        }
    }
}
