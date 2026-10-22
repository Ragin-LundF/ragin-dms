package dms.ui.dashboard

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.FileDropZone
import dms.ui.dashboard.section.DashboardOverview
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.YearFilterModel
import dms.ui.review.ReviewDialog

/**
 * The home screen: the main actions, the headline figures, quick access by category, tag and
 * correspondent, the open tasks, and charts of the library. [onShowAllTasks] opens the tasks screen.
 * Files dropped on it go to the top level.
 */
@Composable
fun DashboardScreen(
    viewModel: DashboardViewModel,
    platform: PlatformActions,
    onAddFiles: (List<String>) -> Unit,
    onChooseFiles: () -> Unit,
    onOpen: (LibrarySelection, String) -> Unit,
    onOpenYear: (YearFilterModel) -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onShowAllTasks: () -> Unit,
    onOpenInbox: () -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    FileDropZone(
        platform = platform,
        label = "The files go to the top level of every library folder",
        onDrop = onAddFiles,
        modifier = Modifier.fillMaxSize(),
    ) {
        DashboardOverview(
            state = state,
            busyReason = busyReason,
            viewModel = viewModel,
            onChooseFiles = onChooseFiles,
            onOpen = onOpen,
            onOpenYear = onOpenYear,
            onOpenDocument = onOpenDocument,
            onShowAllTasks = onShowAllTasks,
            onOpenInbox = onOpenInbox,
        )
    }
    DialogHost(controller = viewModel.dialogs)
    ReviewDialog(controller = viewModel.review, onOpenDocument = viewModel::openDocument)
}
