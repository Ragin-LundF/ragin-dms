package dms.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.window.FrameWindowScope
import androidx.compose.ui.window.MenuBar
import dms.ui.model.Screen
import dms.ui.platform.HostOs
import dms.ui.session.LibrarySession
import dms.ui.session.model.LibraryStatus

/** The native menu bar with the main actions and their shortcuts (⌘ on macOS, Ctrl elsewhere). */
@Composable
fun FrameWindowScope.AppMenuBar(
    session: LibrarySession,
    logVisible: Boolean,
    onScreenChange: (Screen) -> Unit,
    onLogVisibleChange: (Boolean) -> Unit,
) {
    val state by session.state.collectAsState()
    val idle = state.busy == null && state.status is LibraryStatus.Open
    MenuBar {
        Menu(text = "Library") {
            Item(text = "Scan now", enabled = idle, shortcut = HostOs.shortcut(key = Key.R), onClick = session::scan)
            Item(
                text = "Sync to all destinations",
                enabled = idle,
                shortcut = HostOs.shortcut(key = Key.S, shift = true),
                onClick = session::syncToAllDestinations,
            )
            Item(text = "Merge all folders", enabled = idle, onClick = session::mergeAllFolders)
            Item(text = "Retry failed operations", enabled = idle, onClick = session::retryFailedOperations)
            Separator()
            Item(
                text = "Cancel ${state.busy.orEmpty()}".trim(),
                enabled = state.cancellable,
                shortcut = HostOs.shortcut(key = Key.Period),
                onClick = session::cancel,
            )
        }
        Menu(text = "View") {
            Item(
                text = Screen.DASHBOARD.label,
                shortcut = HostOs.shortcut(key = Key.One),
                onClick = { onScreenChange(Screen.DASHBOARD) },
            )
            Item(
                text = Screen.LIBRARY.label,
                shortcut = HostOs.shortcut(key = Key.Two),
                onClick = { onScreenChange(Screen.LIBRARY) },
            )
            Item(
                text = Screen.SETTINGS.label,
                shortcut = HostOs.shortcut(key = Key.Comma),
                onClick = { onScreenChange(Screen.SETTINGS) },
            )
            Separator()
            CheckboxItem(
                text = "Show log",
                checked = logVisible,
                shortcut = HostOs.shortcut(key = Key.L),
                onCheckedChange = onLogVisibleChange,
            )
        }
    }
}
