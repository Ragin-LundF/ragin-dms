package dms.ui.shell.navigation

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import dms.ui.components.ReasonTooltip
import dms.ui.model.Screen
import dms.ui.shell.navigation.model.NavBadgeModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * One screen in the navigation; the selected one sits on a surface pill with a primary icon. A
 * [compact] item shows only its icon, names the screen on hover, and shows its [badge] as a dot.
 */
@Composable
fun NavItem(screen: Screen, selected: Boolean, compact: Boolean, badge: NavBadgeModel?, onClick: () -> Unit) {
    val hover = listOfNotNull(screen.label, badge?.let { "${it.count}" }).joinToString(separator = ": ")
    ReasonTooltip(reason = hover.takeIf { compact }, beside = true) {
        NavItemRow(screen = screen, selected = selected, compact = compact, badge = badge, onClick = onClick)
    }
}

@Composable
private fun NavItemRow(
    screen: Screen,
    selected: Boolean,
    compact: Boolean,
    badge: NavBadgeModel?,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.small
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = shape)
            .background(color = if (selected) colors.surface else Color.Transparent)
            .border(
                width = Spacing.hairline,
                color = if (selected) colors.outlineVariant else Color.Transparent,
                shape = shape,
            )
            .clickable(onClick = onClick)
            .padding(horizontal = Spacing.medium, vertical = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium, alignment = Alignment.CenterHorizontally),
    ) {
        Box {
            Icon(
                imageVector = screen.icon,
                contentDescription = null,
                tint = if (selected) colors.primary else colors.onSurfaceVariant,
                modifier = Modifier.size(size = Spacing.icon),
            )
            if (compact && badge != null) {
                Box(
                    modifier = Modifier
                        .align(alignment = Alignment.TopEnd)
                        .size(size = Spacing.small)
                        .background(color = badgeColor(badge = badge), shape = CircleShape),
                )
            }
        }
        if (!compact) {
            Text(
                text = screen.label,
                style = MaterialTheme.typography.labelLarge,
                color = if (selected) colors.onSurface else colors.onSurfaceVariant,
                modifier = Modifier.weight(weight = 1f),
            )
            badge?.let { CountBadge(badge = it) }
        }
    }
}

@Composable
private fun CountBadge(badge: NavBadgeModel) {
    val color = badgeColor(badge = badge)
    Text(
        text = badge.count.toString(),
        style = MaterialTheme.typography.labelSmall,
        color = color,
        modifier = Modifier
            .background(color = color.copy(alpha = BADGE_ALPHA), shape = CircleShape)
            .padding(horizontal = Spacing.small, vertical = Spacing.hairline),
    )
}

@Composable
private fun badgeColor(badge: NavBadgeModel): Color {
    return if (badge.warning) AppColors.statusWarning else MaterialTheme.colorScheme.primary
}

private const val BADGE_ALPHA = 0.14f
