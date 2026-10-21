package dms.ui.dashboard.task

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
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
    Column(modifier = modifier.padding(start = Spacing.large)) {
        SectionHeader(title = "Open tasks ($total)")
        Row(
            modifier = Modifier.horizontalScroll(state = rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            FilterChip(selected = filter == null, onClick = { onFilter(null) }, label = { Text(text = "All") })
            groups.forEach { group ->
                FilterChip(
                    selected = filter == group.group,
                    onClick = { onFilter(group.group) },
                    label = { Text(text = "${group.group.label} (${group.rows.size})", softWrap = false) },
                )
            }
        }
        if (total == 0) {
            Text(text = "Nothing waits for you.", modifier = Modifier.padding(top = Spacing.large))
        }
        LazyColumn(modifier = Modifier.fillMaxSize()) {
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
                    TaskRowView(row = row, busyReason = busyReason)
                }
            }
        }
    }
}
