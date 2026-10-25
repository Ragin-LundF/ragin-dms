package dms.ui

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.input.pointer.PointerIcon
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.LocalWindowExceptionHandlerFactory
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import dms.storage.platform.AppDataDirectory
import dms.ui.components.layout.LocalResizePointerIcon
import dms.ui.library.LibraryCommands
import dms.ui.model.Screen
import dms.ui.platform.DesktopPlatformActions
import dms.ui.platform.HostOs
import dms.ui.platform.MacTitleBar
import dms.ui.session.LibrarySession
import dms.ui.shell.LocalCommandKey
import dms.ui.shell.LocalTitleBarInset
import dms.ui.shell.palette.CommandPalette
import dms.ui.system.AppErrorHandler
import dms.ui.system.AppIcon
import dms.ui.theme.LocalAppLogo
import java.awt.Cursor

@OptIn(ExperimentalComposeUiApi::class)
fun main() {
    val container = AppContainer(appDataDirectory = AppDataDirectory.forApplication(name = "ragin-dms"))
    val platform = DesktopPlatformActions()
    AppIcon.installInDock()
    application {
        val scope = rememberCoroutineScope()
        val session = remember { LibrarySession(services = container.services, scope = scope) }
        val libraryCommands = remember { LibraryCommands() }
        val palette = remember { CommandPalette(services = container.services, session = session, scope = scope) }
        var screen by remember { mutableStateOf(value = Screen.DASHBOARD) }
        var logVisible by remember { mutableStateOf(value = false) }
        CompositionLocalProvider(LocalWindowExceptionHandlerFactory provides AppErrorHandler) {
            Window(
                onCloseRequest = ::exitApplication,
                title = "Ragin-DMS",
                icon = AppIcon.painter,
                state = rememberWindowState(size = DpSize(width = 1360.dp, height = 860.dp)),
            ) {
                LaunchedEffect(key1 = window) { MacTitleBar.applyTo(window = window) }
                AppMenuBar(
                    session = session,
                    logVisible = logVisible,
                    onScreenChange = { screen = it },
                    onLogVisibleChange = { logVisible = it },
                    onGoTo = { palette.open() },
                    libraryCommands = libraryCommands,
                )
                CompositionLocalProvider(
                    LocalAppLogo provides AppIcon.painter,
                    LocalResizePointerIcon provides PointerIcon(cursor = Cursor(Cursor.E_RESIZE_CURSOR)),
                    LocalTitleBarInset provides MacTitleBar.inset,
                    LocalCommandKey provides HostOs.commandLabel,
                ) {
                    App(
                        services = container.services,
                        platform = platform,
                        session = session,
                        screen = screen,
                        onScreenChange = { screen = it },
                        logVisible = logVisible,
                        onLogVisibleChange = { logVisible = it },
                        palette = palette,
                        libraryCommands = libraryCommands,
                    )
                }
            }
        }
    }
}
