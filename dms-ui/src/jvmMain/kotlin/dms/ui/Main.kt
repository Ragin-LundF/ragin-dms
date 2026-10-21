package dms.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import dms.storage.platform.AppDataDirectory
import dms.ui.model.Screen
import dms.ui.platform.DesktopPlatformActions
import dms.ui.session.LibrarySession

fun main() {
    val container = AppContainer(appDataDirectory = AppDataDirectory.forApplication(name = "ragin-dms"))
    val platform = DesktopPlatformActions()
    application {
        val scope = rememberCoroutineScope()
        val session = remember { LibrarySession(services = container.services, scope = scope) }
        var screen by remember { mutableStateOf(value = Screen.DASHBOARD) }
        Window(
            onCloseRequest = ::exitApplication,
            title = "ragin-dms",
            state = rememberWindowState(size = DpSize(width = 1360.dp, height = 860.dp)),
        ) {
            AppMenuBar(session = session, onScreenChange = { screen = it })
            App(
                services = container.services,
                platform = platform,
                session = session,
                screen = screen,
                onScreenChange = { screen = it },
            )
        }
    }
}
