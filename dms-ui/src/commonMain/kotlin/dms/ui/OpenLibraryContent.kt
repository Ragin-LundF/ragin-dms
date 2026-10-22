package dms.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.lifecycle.viewmodel.compose.viewModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.ui.ask.AskScreen
import dms.ui.ask.AskViewModel
import dms.ui.dashboard.DashboardScreen
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.TasksScreen
import dms.ui.document.InspectorViewModel
import dms.ui.inbox.InboxScreen
import dms.ui.inbox.InboxViewModel
import dms.ui.library.LibraryScreen
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection
import dms.ui.model.Screen
import dms.ui.restructure.RestructureScreen
import dms.ui.restructure.RestructureViewModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.settings.SettingsScreen
import dms.ui.settings.SettingsViewModel

/** The chosen screen of an open library; the dashboard opens the library at what the user clicked. */
@Composable
fun OpenLibraryContent(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    screen: Screen,
    settings: SettingsViewModel,
    onScreenChange: (Screen) -> Unit,
) {
    val library = viewModel { LibraryViewModel(services = services, session = session, platform = platform) }
    val dashboard = viewModel { DashboardViewModel(services = services, session = session, platform = platform) }
    val restructure = viewModel { RestructureViewModel(services = services, session = session) }
    val restructureRequest by dashboard.organize.restructureRequests.collectAsState()
    LaunchedEffect(key1 = restructureRequest) {
        val folder = restructureRequest ?: return@LaunchedEffect
        dashboard.organize.consumeRestructureRequest()
        restructure.start(folder = folder)
        onScreenChange(Screen.RESTRUCTURE)
    }
    val openDocument: (DocumentIdModel) -> Unit = { documentId ->
        library.reveal(documentId = documentId)
        onScreenChange(Screen.LIBRARY)
    }
    when (screen) {
        Screen.DASHBOARD -> DashboardScreen(
            viewModel = dashboard,
            platform = platform,
            onAddFiles = { files -> library.addFiles(locations = files, target = RelativePathModel.ROOT) },
            onChooseFiles = library::addFilesToTopLevel,
            onOpen = { selection, query ->
                library.show(selection = selection, query = query)
                onScreenChange(Screen.LIBRARY)
            },
            onOpenYear = { year ->
                library.show(
                    selection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
                    query = "",
                    year = year,
                )
                onScreenChange(Screen.LIBRARY)
            },
            onOpenDocument = openDocument,
            onShowAllTasks = { onScreenChange(Screen.TASKS) },
            onOpenInbox = { onScreenChange(Screen.INBOX) },
        )

        Screen.INBOX -> InboxScreen(
            viewModel = viewModel { InboxViewModel(services = services, session = session) },
            onOpenSettings = { onScreenChange(Screen.SETTINGS) },
            onOpenDocument = openDocument,
        )

        Screen.ASK -> AskScreen(
            viewModel = viewModel { AskViewModel(services = services, session = session) },
            onOpenDocument = openDocument,
        )

        Screen.RESTRUCTURE -> RestructureScreen(viewModel = restructure)

        Screen.TASKS -> TasksScreen(viewModel = dashboard)

        Screen.LIBRARY -> LibraryScreen(
            viewModel = library,
            inspector = viewModel { InspectorViewModel(services = services, session = session, platform = platform) },
            platform = platform,
        )

        Screen.SETTINGS -> SettingsScreen(viewModel = settings)
    }
}
