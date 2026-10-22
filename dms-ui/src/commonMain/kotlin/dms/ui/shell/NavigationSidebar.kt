package dms.ui.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.model.Screen
import dms.ui.session.model.SessionState
import dms.ui.theme.Spacing

/**
 * The app's left column: the brand, the screens, and what the library is doing right now. While
 * [collapsed] it shows only icons, which name themselves on hover.
 */
@Composable
fun NavigationSidebar(
    screen: Screen,
    sessionState: SessionState,
    collapsed: Boolean,
    onToggleCollapsed: () -> Unit,
    onScreenChange: (Screen) -> Unit,
) {
    val width = if (collapsed) Spacing.navigationCollapsedWidth else Spacing.navigationWidth
    Surface(
        modifier = Modifier.width(width = width).fillMaxHeight(),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
    ) {
        Column(
            modifier = Modifier.padding(all = Spacing.large),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            BrandMark(compact = collapsed)
            Spacer(modifier = Modifier.padding(top = Spacing.large))
            Screen.entries.forEach { entry ->
                NavItem(screen = entry, selected = entry == screen, compact = collapsed) { onScreenChange(entry) }
            }
            Spacer(modifier = Modifier.weight(weight = 1f))
            StatusFooter(sessionState = sessionState, compact = collapsed)
            NavCollapseButton(collapsed = collapsed, onToggle = onToggleCollapsed)
        }
    }
}
