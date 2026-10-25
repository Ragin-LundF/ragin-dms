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
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.dashboard.DashboardViewModel
import dms.ui.library.LibraryViewModel
import dms.ui.model.Screen
import dms.ui.session.LibrarySession
import dms.ui.shell.LocalCommandKey
import dms.ui.shell.palette.CommandPalette
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
    val command = LocalCommandKey.current
    val title = if (screen == Screen.LIBRARY) libraryState.title else screen.label
    WindowToolbar(title = title, onOpenPalette = { palette.open() }) {
        when (screen) {
            Screen.DASHBOARD -> {
                SplitButton(
                    main = ActionButtonModel(label = "Scan", icon = Icons.Rounded.Refresh, onClick = dashboard::scan),
                    menu = listOf(
                        ActionButtonModel(
                            label = "Sync to all",
                            icon = Icons.Rounded.Sync,
                            shortcut = "$command⇧S",
                            onClick = dashboard::syncToAllDestinations,
                        ),
                        ActionButtonModel(
                            label = "Merge all folders…",
                            icon = Icons.AutoMirrored.Rounded.MergeType,
                            onClick = dashboard::mergeAllFolders,
                        ),
                        ActionButtonModel(
                            label = "Tag untagged with AI",
                            icon = Icons.Rounded.AutoAwesome,
                            disabledReason = sessionState.aiBusy,
                            onClick = dashboard.aiBatches::tagAll,
                        ),
                    ),
                    menuLabel = "More library actions",
                    busyReason = busy,
                )
                AddFilesButton(busyReason = busy, onClick = library::addFilesToTopLevel)
            }

            Screen.LIBRARY -> AddFilesButton(busyReason = busy, onClick = library::addFiles)

            Screen.INBOX,
            Screen.TASKS,
            Screen.ASK,
            Screen.RESTRUCTURE,
            Screen.SETTINGS,
            -> Unit
        }
    }
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
