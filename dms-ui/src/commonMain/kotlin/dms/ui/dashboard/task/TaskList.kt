package dms.ui.dashboard.task

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.TaskAlt
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.components.surface.EmptyState
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskGroupState
import dms.ui.theme.Spacing

/** The open tasks, grouped by kind, with a filter per kind. */
@Composable
fun TaskList(
    groups: List<TaskGroupState>,
    filter: TaskGroup?,
    busyReason: String?,
    onFilter: (TaskGroup?) -> Unit,
    modifier: Modifier = Modifier,
) {
    val total = groups.sumOf { it.rows.size }
    AppCard(modifier = modifier) {
        CardTitle(
            title = "Open tasks",
            icon = Icons.Rounded.TaskAlt,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = if (total == 0) "Nothing waits for you" else "$total waiting for your decision",
        )
        if (total == 0) {
            EmptyState(
                icon = Icons.Rounded.TaskAlt,
                title = "All caught up",
                message = "New, missing and changed files appear here after a scan.",
                modifier = Modifier.fillMaxSize(),
            )
            return@AppCard
        }
        Row(
            modifier = Modifier.horizontalScroll(state = rememberScrollState()).padding(bottom = Spacing.small),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            FilterChip(selected = filter == null, onClick = { onFilter(null) }, label = { Text(text = "All ($total)") })
            groups.forEach { group ->
                FilterChip(
                    selected = filter == group.group,
                    onClick = { onFilter(group.group) },
                    label = { Text(text = "${group.group.label} (${group.rows.size})", softWrap = false) },
                )
            }
        }
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            groups.filter { filter == null || it.group == filter }.forEach { group ->
                item(key = "group:${group.group.name}") {
                    SectionHeader(title = group.group.label) {
                        group.bulkAction?.let { action ->
                            ActionButtons(
                                buttons = listOf(action),
                                enabled = busyReason == null,
                                busyReason = busyReason,
                            )
                        }
                    }
                }
                items(items = group.rows, key = { it.key.value }) { row ->
                    TaskRowView(row = row, group = group.group, busyReason = busyReason)
                }
            }
        }
    }
}
