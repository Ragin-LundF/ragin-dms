package dms.ui.dashboard

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.domain.document.DocumentId
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.FileDropZone
import dms.ui.dashboard.section.DashboardOverview
import dms.ui.dashboard.task.TaskList
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.YearFilter
import dms.ui.review.ReviewDialog
import dms.ui.theme.Spacing

/**
 * The home screen: the main actions, the headline figures, quick access by category, tag and
 * correspondent, the open tasks, and charts of the library. The open tasks can fill the screen.
 * Files dropped on it go to the top level.
 */
@Composable
fun DashboardScreen(
    viewModel: DashboardViewModel,
    platform: PlatformActions,
    onAddFiles: (List<String>) -> Unit,
    onChooseFiles: () -> Unit,
    onOpen: (LibrarySelection, String) -> Unit,
    onOpenYear: (YearFilter) -> Unit,
    onOpenDocument: (DocumentId) -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    var tasksFull by remember { mutableStateOf(value = false) }
    FileDropZone(
        platform = platform,
        label = "The files go to the top level of every library folder",
        onDrop = onAddFiles,
        modifier = Modifier.fillMaxSize(),
    ) {
        if (tasksFull) {
            TaskList(
                groups = state.groups,
                filter = state.filter,
                busyReason = busyReason,
                thumbnails = viewModel.thumbnails,
                onFilter = viewModel::setFilter,
                full = true,
                onFullView = { tasksFull = false },
                modifier = Modifier.fillMaxSize().padding(all = Spacing.xxlarge),
            )
        } else {
            DashboardOverview(
                state = state,
                busyReason = busyReason,
                viewModel = viewModel,
                onChooseFiles = onChooseFiles,
                onOpen = onOpen,
                onOpenYear = onOpenYear,
                onOpenDocument = onOpenDocument,
                onShowAllTasks = { tasksFull = true },
            )
        }
    }
    DialogHost(controller = viewModel.dialogs)
    ReviewDialog(controller = viewModel.review, onOpenDocument = viewModel::openDocument)
}
