package dms.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.ui.model.Screen
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.shell.ActivityBar
import dms.ui.shell.LogPanel
import dms.ui.shell.navigation.NavigationCounts
import dms.ui.shell.navigation.NavigationSidebar
import dms.ui.shell.palette.CommandPalette

/** The navigation on the left, the screen, the log and the activity bar below it, and the messages. */
@Composable
fun AppContent(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    screen: Screen,
    onScreenChange: (Screen) -> Unit,
    logVisible: Boolean,
    onLogVisibleChange: (Boolean) -> Unit,
    palette: CommandPalette,
) {
    val sessionState by session.state.collectAsState()
    val progress by session.progress.collectAsState()
    val snackbar = remember { SnackbarHostState() }
    var navigationCollapsed by remember { mutableStateOf(value = false) }
    val scope = rememberCoroutineScope()
    val navigationCounts = remember { NavigationCounts(services = services, session = session, scope = scope) }
    val counts by navigationCounts.counts.collectAsState()
    LaunchedEffect(key1 = sessionState.message) {
        val message = sessionState.message ?: return@LaunchedEffect
        snackbar.showSnackbar(
            message = message.text,
            withDismissAction = true,
            duration = if (message.isError) SnackbarDuration.Long else SnackbarDuration.Short,
        )
        session.clearMessage()
    }
    Scaffold(
        snackbarHost = { SnackbarHost(hostState = snackbar) },
        containerColor = MaterialTheme.colorScheme.background,
    ) { padding ->
        Row(modifier = Modifier.fillMaxSize().padding(paddingValues = padding)) {
            NavigationSidebar(
                screen = screen,
                counts = counts,
                // The library needs the width for its own sidebar, so the navigation is a rail there.
                collapsed = navigationCollapsed || screen == Screen.LIBRARY,
                onToggleCollapsed = { navigationCollapsed = !navigationCollapsed },
                onScreenChange = onScreenChange,
            )
            Column(modifier = Modifier.weight(weight = 1f).fillMaxHeight()) {
                Box(modifier = Modifier.weight(weight = 1f).fillMaxWidth()) {
                    ScreenContent(
                        services = services,
                        platform = platform,
                        session = session,
                        status = sessionState.status,
                        screen = screen,
                        onScreenChange = onScreenChange,
                        palette = palette,
                    )
                }
                if (logVisible) {
                    val entries by session.log.entries.collectAsState()
                    LogPanel(entries = entries, onClear = session.log::clear)
                }
                ActivityBar(
                    sessionState = sessionState,
                    progress = progress,
                    logVisible = logVisible,
                    onCancel = session::cancel,
                    onToggleLog = { onLogVisibleChange(!logVisible) },
                )
            }
        }
    }
}
