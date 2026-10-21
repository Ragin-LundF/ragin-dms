package dms.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import dms.ui.model.Screen
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.LibraryStatus
import dms.ui.settings.SettingsScreen
import dms.ui.settings.SettingsViewModel

/** The navigation rail, the busy line, the messages, and the screen for the library's status. */
@Composable
fun AppContent(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    screen: Screen,
    onScreenChange: (Screen) -> Unit,
) {
    val sessionState by session.state.collectAsState()
    val snackbar = remember { SnackbarHostState() }
    LaunchedEffect(key1 = sessionState.message) {
        val message = sessionState.message ?: return@LaunchedEffect
        snackbar.showSnackbar(
            message = message.text,
            withDismissAction = true,
            duration = if (message.isError) SnackbarDuration.Long else SnackbarDuration.Short,
        )
        session.clearMessage()
    }
    Scaffold(snackbarHost = { SnackbarHost(hostState = snackbar) }) { padding ->
        Row(modifier = Modifier.fillMaxSize().padding(paddingValues = padding)) {
            NavigationRail {
                Screen.entries.forEach { entry ->
                    NavigationRailItem(
                        selected = entry == screen,
                        onClick = { onScreenChange(entry) },
                        icon = { Text(text = entry.symbol) },
                        label = { Text(text = entry.label) },
                    )
                }
            }
            Column(modifier = Modifier.fillMaxSize()) {
                BusyLine(busy = sessionState.busy)
                val settings = viewModel {
                    SettingsViewModel(services = services, session = session, platform = platform)
                }
                when (val status = sessionState.status) {
                    LibraryStatus.Loading -> Box(
                        modifier = Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center,
                    ) {
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
                    )
                }
            }
        }
    }
}
