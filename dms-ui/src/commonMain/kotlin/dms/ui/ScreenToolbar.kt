package dms.ui

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.MergeType
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Sync
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.vector.ImageVector
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.dashboard.DashboardViewModel
import dms.ui.library.LibraryViewModel
import dms.ui.model.Screen
import dms.ui.session.LibrarySession
import dms.ui.shell.LocalCommandKey
import dms.ui.session.OutcomeMessages
import dms.ui.shell.palette.CommandPalette
import dms.ui.shell.palette.model.PaletteAction
import dms.ui.shell.toolbar.SplitButton
import dms.ui.shell.toolbar.WindowToolbar

/**
 * The window toolbar of [screen]: its title, the palette field and its actions. The dashboard gets the
 * Scan split button and Add files, the library Add files; the other screens only their title.
 */
@Composable
fun ScreenToolbar(
    screen: Screen,
    session: LibrarySession,
    dashboard: DashboardViewModel,
    library: LibraryViewModel,
    palette: CommandPalette,
) {
    val sessionState by session.state.collectAsState()
    val libraryState by library.state.collectAsState()
    val busy = sessionState.busy
    val title = if (screen == Screen.LIBRARY) libraryState.title else screen.label
    WindowToolbar(title = title, onOpenPalette = { palette.open() }) {
        when (screen) {
            Screen.DASHBOARD -> {
                SplitButton(
                    main = ActionButtonModel(label = "Scan", icon = Icons.Rounded.Refresh, onClick = dashboard::scan),
                    menu = listOf(
                        menuItem(action = PaletteAction.SYNC_ALL, icon = Icons.Rounded.Sync, waitingFor = busy) {
                            dashboard.syncToAllDestinations()
                        },
                        menuItem(
                            action = PaletteAction.MERGE_ALL,
                            icon = Icons.AutoMirrored.Rounded.MergeType,
                            waitingFor = busy,
                        ) { dashboard.mergeAllFolders() },
                        menuItem(
                            action = PaletteAction.TAG_UNTAGGED,
                            icon = Icons.Rounded.AutoAwesome,
                            waitingFor = sessionState.aiBusy,
                        ) { dashboard.aiBatches.tagAll() }.takeIf { sessionState.aiEnabled },
                    ).filterNotNull(),
                    menuLabel = "More library actions",
                    busyReason = busy,
                )
                AddFilesButton(busyReason = busy, onClick = library.files::addFilesToTopLevel)
            }

            Screen.LIBRARY -> AddFilesButton(busyReason = busy, onClick = library.files::addFiles)

            Screen.INBOX,
            Screen.TASKS,
            Screen.ASK,
            Screen.RESTRUCTURE,
            Screen.SETTINGS,
            -> Unit
        }
    }
}

/**
 * A menu item for [action], with its label and shortcut from the palette's list, so menu bar, palette and
 * toolbar name it alike; it waits while [waitingFor] names what runs in its lane.
 */
@Composable
private fun menuItem(
    action: PaletteAction,
    icon: ImageVector,
    waitingFor: String?,
    onClick: () -> Unit,
): ActionButtonModel {
    val command = LocalCommandKey.current
    return ActionButtonModel(
        label = action.label,
        icon = icon,
        disabledReason = waitingFor?.let { OutcomeMessages.waitFor(operation = it) },
        shortcut = action.shortcutKey?.let { key -> command + (if (action.shift) "⇧" else "") + key },
        onClick = onClick,
    )
}

@Composable
private fun AddFilesButton(busyReason: String?, onClick: () -> Unit) {
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(
                label = "Add files",
                icon = Icons.Rounded.Add,
                style = ButtonStyle.FILLED,
                onClick = onClick,
            ),
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
