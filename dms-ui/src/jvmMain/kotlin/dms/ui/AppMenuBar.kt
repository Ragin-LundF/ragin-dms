package dms.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.window.FrameWindowScope
import androidx.compose.ui.window.MenuBar
import dms.ui.model.Screen
import dms.ui.platform.ActionShortcuts
import dms.ui.platform.HostOs
import dms.ui.session.LibrarySession
import dms.ui.session.model.LibraryStatus
import dms.ui.shell.palette.model.PaletteAction

/**
 * The native menu bar with the main actions and their shortcuts (⌘ on macOS, Ctrl elsewhere); "Go to…"
 * opens the command palette.
 */
@Composable
fun FrameWindowScope.AppMenuBar(
    session: LibrarySession,
    logVisible: Boolean,
    onScreenChange: (Screen) -> Unit,
    onLogVisibleChange: (Boolean) -> Unit,
    onGoTo: () -> Unit,
) {
    val state by session.state.collectAsState()
    val idle = state.busy == null && state.status is LibraryStatus.Open
    val libraryActions = listOf(
        PaletteAction.SCAN to session::scan,
        PaletteAction.SYNC_ALL to session::syncToAllDestinations,
        PaletteAction.MERGE_ALL to session::mergeAllFolders,
        PaletteAction.RETRY_FAILED to session::retryFailedOperations,
        PaletteAction.DETECT_DATES to session::detectDocumentDates,
    )
    val screens = listOf(
        PaletteAction.OPEN_DASHBOARD to Screen.DASHBOARD,
        PaletteAction.OPEN_TASKS to Screen.TASKS,
        PaletteAction.OPEN_LIBRARY to Screen.LIBRARY,
        PaletteAction.OPEN_SETTINGS to Screen.SETTINGS,
    )
    MenuBar {
        Menu(text = "Library") {
            libraryActions.forEach { (action, run) ->
                Item(text = action.label, enabled = idle, shortcut = ActionShortcuts.of(action = action), onClick = run)
            }
            Separator()
            val cancellable = state.operations.firstOrNull { it.cancellable }
            Item(
                text = "Cancel ${cancellable?.label.orEmpty()}".trim(),
                enabled = cancellable != null,
                shortcut = HostOs.shortcut(key = Key.Period),
                onClick = { cancellable?.let { session.cancel(lane = it.lane) } },
            )
        }
        Menu(text = "View") {
            Item(text = "Go to…", shortcut = HostOs.shortcut(key = Key.K), onClick = onGoTo)
            Separator()
            screens.forEach { (action, screen) ->
                Item(
                    text = screen.label,
                    shortcut = ActionShortcuts.of(action = action),
                    onClick = { onScreenChange(screen) },
                )
            }
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
