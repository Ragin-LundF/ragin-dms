package dms.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.ui.ask.AskScreen
import dms.ui.ask.AskViewModel
import dms.ui.dashboard.DashboardScreen
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.TasksScreen
import dms.ui.document.QuickLookViewModel
import dms.ui.inbox.InboxScreen
import dms.ui.inbox.InboxViewModel
import dms.ui.library.LibraryCommands
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
import dms.ui.shell.palette.CommandPalette
import dms.ui.shell.palette.CommandPaletteDialog
import dms.ui.shell.palette.PaletteCommands

/**
 * The chosen screen of an open library under the window toolbar, and the command palette; the
 * dashboard and the palette open the library at what the user chose.
 */
@Composable
fun OpenLibraryContent(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    screen: Screen,
    settings: SettingsViewModel,
    onScreenChange: (Screen) -> Unit,
    palette: CommandPalette,
    libraryCommands: LibraryCommands,
) {
    val quickLookViewModel = viewModel {
        QuickLookViewModel(services = services, session = session, platform = platform)
    }
    // One quick look for the whole library: the documents, the folder view and the keys all open the same.
    val library = viewModel {
        LibraryViewModel(
            services = services,
            session = session,
            platform = platform,
            quickLook = quickLookViewModel.quickLook,
        )
    }
    val dashboard = viewModel { DashboardViewModel(services = services, session = session, platform = platform) }
    ConnectedCommands(commands = libraryCommands, library = library)
    val restructure = viewModel { RestructureViewModel(services = services, session = session) }
    val ask = viewModel { AskViewModel(services = services, session = session) }
    RestructureRequests(dashboard = dashboard, restructure = restructure, onScreenChange = onScreenChange)
    val openDocument: (DocumentIdModel) -> Unit = { documentId ->
        library.reveal(documentId = documentId)
        onScreenChange(Screen.LIBRARY)
    }
    val commands = remember(library, dashboard, ask) {
        PaletteCommands(
            session = session,
            dashboard = dashboard,
            library = library,
            ask = ask,
            onScreenChange = onScreenChange,
            onOpenDocument = openDocument,
        )
    }
    CommandPaletteDialog(palette = palette, onChoose = commands::run)
    Column(modifier = Modifier.fillMaxSize()) {
        ScreenToolbar(screen = screen, session = session, dashboard = dashboard, library = library, palette = palette)
        Box(modifier = Modifier.weight(weight = 1f).fillMaxWidth()) {
            when (screen) {
                Screen.DASHBOARD -> Dashboard(
                    dashboard = dashboard,
                    library = library,
                    platform = platform,
                    onScreenChange = onScreenChange,
                    onOpenDocument = openDocument,
                )

                Screen.INBOX -> InboxScreen(
                    viewModel = viewModel { InboxViewModel(services = services, session = session) },
                    onOpenSettings = { onScreenChange(Screen.SETTINGS) },
                    onOpenDocument = openDocument,
                )

                Screen.ASK -> AskScreen(viewModel = ask, onOpenDocument = openDocument)
                Screen.RESTRUCTURE -> RestructureScreen(viewModel = restructure)
                Screen.TASKS -> TasksScreen(viewModel = dashboard)
                Screen.LIBRARY ->
                    LibraryScreen(viewModel = library, quickLookViewModel = quickLookViewModel, platform = platform)

                Screen.SETTINGS -> SettingsScreen(viewModel = settings)
            }
        }
    }
}

/** Lets the "Documents" menu reach [library] while it is shown. */
@Composable
private fun ConnectedCommands(commands: LibraryCommands, library: LibraryViewModel) {
    DisposableEffect(key1 = library) {
        commands.connect(viewModel = library)
        onDispose { commands.connect(viewModel = null) }
    }
}

@Composable
private fun Dashboard(
    dashboard: DashboardViewModel,
    library: LibraryViewModel,
    platform: PlatformActions,
    onScreenChange: (Screen) -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
) {
    DashboardScreen(
        viewModel = dashboard,
        platform = platform,
        onAddFiles = { files -> library.files.addFiles(locations = files, target = RelativePathModel.ROOT) },
        onOpen = { selection, query ->
            library.show(selection = selection, query = query)
            onScreenChange(Screen.LIBRARY)
        },
        onOpenYear = { year ->
            library.show(selection = LibrarySelection.Folder(path = RelativePathModel.ROOT), query = "", year = year)
            onScreenChange(Screen.LIBRARY)
        },
        onOpenDocument = onOpenDocument,
        onShowAllTasks = { onScreenChange(Screen.TASKS) },
        onOpenInbox = { onScreenChange(Screen.INBOX) },
    )
}

/** Opens the restructure assistant on the folder a dashboard task asks for. */
@Composable
private fun RestructureRequests(
    dashboard: DashboardViewModel,
    restructure: RestructureViewModel,
    onScreenChange: (Screen) -> Unit,
) {
    val restructureRequest by dashboard.organize.restructureRequests.collectAsState()
    LaunchedEffect(key1 = restructureRequest) {
        val folder = restructureRequest ?: return@LaunchedEffect
        dashboard.organize.consumeRestructureRequest()
        restructure.start(folder = folder)
        onScreenChange(Screen.RESTRUCTURE)
    }
}
