package dms.ui

import androidx.compose.runtime.Composable
import androidx.lifecycle.viewmodel.compose.viewModel
import dms.domain.library.RelativePath
import dms.ui.dashboard.DashboardScreen
import dms.ui.dashboard.DashboardViewModel
import dms.ui.document.InspectorViewModel
import dms.ui.library.LibraryScreen
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection
import dms.ui.model.Screen
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
    when (screen) {
        Screen.DASHBOARD -> DashboardScreen(
            viewModel = viewModel { DashboardViewModel(services = services, session = session) },
            platform = platform,
            onAddFiles = { files -> library.addFiles(locations = files, target = RelativePath.ROOT) },
            onChooseFiles = library::addFilesToTopLevel,
            onOpen = { selection, query ->
                library.show(selection = selection, query = query)
                onScreenChange(Screen.LIBRARY)
            },
            onOpenYear = { year ->
                library.show(selection = LibrarySelection.Folder(path = RelativePath.ROOT), query = "", year = year)
                onScreenChange(Screen.LIBRARY)
            },
            onOpenDocument = { documentId ->
                library.reveal(documentId = documentId)
                onScreenChange(Screen.LIBRARY)
            },
        )

        Screen.LIBRARY -> LibraryScreen(
            viewModel = library,
            inspector = viewModel { InspectorViewModel(services = services, session = session, platform = platform) },
            platform = platform,
        )

        Screen.SETTINGS -> SettingsScreen(viewModel = settings)
    }
}
