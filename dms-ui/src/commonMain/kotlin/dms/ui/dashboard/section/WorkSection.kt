package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.dashboard.ActivityPanel
import dms.ui.dashboard.FolderPanel
import dms.ui.dashboard.model.DashboardUiState
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.task.TaskList
import dms.ui.theme.Spacing

/** What needs the user: the open tasks, next to the state of the folders and what happened lately. */
@Composable
fun WorkSection(state: DashboardUiState, busyReason: String?, onFilter: (TaskGroup?) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().height(height = 560.dp),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        TaskList(
            groups = state.groups,
            filter = state.filter,
            busyReason = busyReason,
            onFilter = onFilter,
            modifier = Modifier.weight(weight = 2f),
        )
        Column(
            modifier = Modifier.weight(weight = 1f),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            FolderPanel(folders = state.folders)
            ActivityPanel(entries = state.activity, modifier = Modifier.weight(weight = 1f))
        }
    }
}
