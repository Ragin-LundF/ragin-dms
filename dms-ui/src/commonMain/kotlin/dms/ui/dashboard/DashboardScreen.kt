package dms.ui.dashboard

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.ui.components.DialogHost
import dms.ui.dashboard.task.TaskList
import dms.ui.theme.Spacing

/** The home screen: folders, open tasks and recent activity. */
@Composable
fun DashboardScreen(viewModel: DashboardViewModel) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    Column(modifier = Modifier.fillMaxSize().padding(all = Spacing.large)) {
        DashboardToolbar(
            lastScanAt = state.lastScanAt,
            busyReason = busyReason,
            onScan = viewModel::scan,
            onSync = viewModel::syncToAllDestinations,
            onMerge = viewModel::mergeAllFolders,
        )
        HorizontalDivider()
        Row(modifier = Modifier.weight(weight = 1f).fillMaxWidth()) {
            FolderPanel(
                folders = state.folders,
                documentCount = state.documentCount,
                uncategorizedCount = state.uncategorizedCount,
                modifier = Modifier.width(width = Spacing.sidebarWidth).fillMaxHeight(),
            )
            VerticalDivider()
            TaskList(
                groups = state.groups,
                filter = state.filter,
                busyReason = busyReason,
                onFilter = viewModel::setFilter,
                modifier = Modifier.weight(weight = 1f).fillMaxHeight(),
            )
        }
        HorizontalDivider()
        ActivityPanel(entries = state.activity)
    }
    DialogHost(controller = viewModel.dialogs)
}
