package dms.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.lifecycle.viewmodel.compose.LocalViewModelStoreOwner
import dms.ui.model.Screen
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.theme.AppTheme

/** Root composable: opens the library once and shows the screen the user chose. */
@Composable
fun App(
    services: AppServices,
    platform: PlatformActions,
    session: LibrarySession,
    screen: Screen,
    onScreenChange: (Screen) -> Unit,
    logVisible: Boolean,
    onLogVisibleChange: (Boolean) -> Unit,
) {
    val owner = remember { AppViewModelStoreOwner() }
    DisposableEffect(key1 = owner) {
        onDispose { owner.viewModelStore.clear() }
    }
    LaunchedEffect(key1 = session) {
        session.open()
    }
    AppTheme {
        CompositionLocalProvider(LocalViewModelStoreOwner provides owner) {
            AppContent(
                services = services,
                platform = platform,
                session = session,
                screen = screen,
                onScreenChange = onScreenChange,
                logVisible = logVisible,
                onLogVisibleChange = onLogVisibleChange,
            )
        }
    }
}
