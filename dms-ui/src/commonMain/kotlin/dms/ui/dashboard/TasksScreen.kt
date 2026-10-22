package dms.ui.dashboard

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.ui.components.dialog.DialogHost
import dms.ui.dashboard.task.TaskList
import dms.ui.review.ReviewDialog
import dms.ui.theme.Spacing

/** Every open task, filling the screen; it shares the dashboard's view model and so its filter. */
@Composable
fun TasksScreen(viewModel: DashboardViewModel) {
    val state by viewModel.state.collectAsState()
    TaskList(
        groups = state.groups,
        filter = state.filter,
        busyReason = state.busy?.let { "Wait until \"$it\" finishes" },
        thumbnails = viewModel.thumbnails,
        onFilter = viewModel::setFilter,
        onFullView = null,
        modifier = Modifier.fillMaxSize().padding(all = Spacing.xxlarge),
    )
    DialogHost(controller = viewModel.dialogs)
    ReviewDialog(controller = viewModel.review, onOpenDocument = viewModel::openDocument)
}
