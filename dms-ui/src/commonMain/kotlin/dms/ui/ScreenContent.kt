package dms.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import dms.ui.model.Screen
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.LibraryStatus
import dms.ui.settings.SettingsScreen
import dms.ui.settings.SettingsViewModel

/** The screen for the library's status: settings until it is configured, the chosen screen once it is open. */
@Composable
fun ScreenContent(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    status: LibraryStatus,
    screen: Screen,
    onScreenChange: (Screen) -> Unit,
) {
    val settings = viewModel {
        SettingsViewModel(services = services, session = session, platform = platform)
    }
    when (status) {
        LibraryStatus.Loading -> Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }

        LibraryStatus.NotConfigured -> SettingsScreen(viewModel = settings)
        is LibraryStatus.PrimaryFailed, is LibraryStatus.Failed -> if (screen == Screen.SETTINGS) {
            SettingsScreen(viewModel = settings)
        } else {
            LibraryProblem(
                status = status,
                onRetry = session::open,
                onOpenSettings = { onScreenChange(Screen.SETTINGS) },
            )
        }

        is LibraryStatus.Open -> OpenLibraryContent(
            services = services,
            platform = platform,
            session = session,
            screen = screen,
            settings = settings,
            onScreenChange = onScreenChange,
        )
    }
}
