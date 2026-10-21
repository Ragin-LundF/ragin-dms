package dms.ui

import androidx.compose.runtime.Composable
import androidx.lifecycle.viewmodel.compose.viewModel
import dms.ui.dashboard.DashboardScreen
import dms.ui.dashboard.DashboardViewModel
import dms.ui.document.InspectorViewModel
import dms.ui.library.LibraryScreen
import dms.ui.library.LibraryViewModel
import dms.ui.model.Screen
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.settings.SettingsScreen
import dms.ui.settings.SettingsViewModel

/** The chosen screen of an open library. */
@Composable
fun OpenLibraryContent(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    screen: Screen,
    settings: SettingsViewModel,
) {
    when (screen) {
        Screen.DASHBOARD -> DashboardScreen(
            viewModel = viewModel { DashboardViewModel(services = services, session = session) },
        )

        Screen.LIBRARY -> LibraryScreen(
            viewModel = viewModel { LibraryViewModel(services = services, session = session, platform = platform) },
            inspector = viewModel { InspectorViewModel(services = services, session = session, platform = platform) },
        )

        Screen.SETTINGS -> SettingsScreen(viewModel = settings)
    }
}
