package dms.ui.dashboard.task

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskGroupState
import dms.ui.theme.Spacing

/** One chip per kind of task, with its count; the selected one filters the list. */
@Composable
fun TaskFilterChips(groups: List<TaskGroupState>, filter: TaskGroup?, total: Int, onFilter: (TaskGroup?) -> Unit) {
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
}
