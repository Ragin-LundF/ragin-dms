package dms.ui.shell.navigation

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.model.NavGroup
import dms.ui.model.Screen
import dms.ui.shell.BrandMark
import dms.ui.shell.LocalTitleBarInset
import dms.ui.shell.navigation.model.NavBadgeModel
import dms.ui.shell.navigation.model.NavigationCountsModel
import dms.ui.theme.Spacing
import java.util.Locale

/**
 * The app's left column: the brand, the screens in their groups with the inbox and task counts, and
 * Settings at the bottom. While [collapsed] it is a rail of icons that name themselves on hover.
 */
@Composable
fun NavigationSidebar(
    screen: Screen,
    counts: NavigationCountsModel,
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
            modifier = Modifier
                .padding(top = LocalTitleBarInset.current)
                .padding(all = Spacing.medium),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
        ) {
            BrandMark(compact = collapsed)
            NavGroup.entries.forEach { group ->
                GroupLabel(group = group, compact = collapsed)
                Screen.entries.filter { it.group == group }.forEach { entry ->
                    NavItem(
                        screen = entry,
                        selected = entry == screen,
                        compact = collapsed,
                        badge = badgeOf(screen = entry, counts = counts),
                    ) { onScreenChange(entry) }
                }
            }
            Spacer(modifier = Modifier.weight(weight = 1f))
            Screen.entries.filter { it.group == null }.forEach { entry ->
                NavItem(screen = entry, selected = entry == screen, compact = collapsed, badge = null) {
                    onScreenChange(entry)
                }
            }
            NavCollapseButton(collapsed = collapsed, onToggle = onToggleCollapsed)
        }
    }
}

@Composable
private fun GroupLabel(group: NavGroup, compact: Boolean) {
    if (compact) {
        Spacer(modifier = Modifier.height(height = Spacing.medium))
        return
    }
    Text(
        text = group.label.uppercase(Locale.ROOT),
        style = MaterialTheme.typography.labelSmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.padding(start = Spacing.medium, top = Spacing.large, bottom = Spacing.xsmall),
    )
}

/** The badge of [screen]; `null` for screens without one or a count of 0. */
private fun badgeOf(screen: Screen, counts: NavigationCountsModel): NavBadgeModel? {
    val badge = when (screen) {
        Screen.INBOX -> NavBadgeModel(count = counts.inbox, warning = false)
        Screen.TASKS -> NavBadgeModel(count = counts.tasks, warning = true)
        Screen.DASHBOARD,
        Screen.LIBRARY,
        Screen.ASK,
        Screen.RESTRUCTURE,
        Screen.SETTINGS,
        -> null
    }
    return badge?.takeIf { it.count > 0 }
}
