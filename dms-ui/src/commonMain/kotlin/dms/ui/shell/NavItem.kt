package dms.ui.shell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dms.ui.components.ReasonTooltip
import dms.ui.model.Screen
import dms.ui.theme.Spacing

/**
 * One screen in the navigation; the selected one sits on a tinted pill. A [compact] item shows only
 * its icon and names the screen on hover.
 */
@Composable
fun NavItem(screen: Screen, selected: Boolean, compact: Boolean, onClick: () -> Unit) {
    ReasonTooltip(reason = screen.label.takeIf { compact }, beside = true) {
        NavItemRow(screen = screen, selected = selected, compact = compact, onClick = onClick)
    }
}

@Composable
private fun NavItemRow(screen: Screen, selected: Boolean, compact: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = MaterialTheme.shapes.medium)
            .background(color = if (selected) colors.primaryContainer else Color.Transparent)
            .clickable(onClick = onClick)
            .padding(horizontal = Spacing.medium, vertical = Spacing.medium),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium, alignment = Alignment.CenterHorizontally),
    ) {
        Icon(
            imageVector = screen.icon,
            contentDescription = null,
            tint = if (selected) colors.primary else colors.onSurfaceVariant,
            modifier = Modifier.size(size = 22.dp),
        )
        if (!compact) {
            Text(
                text = screen.label,
                style = MaterialTheme.typography.labelLarge,
                color = if (selected) colors.onPrimaryContainer else colors.onSurfaceVariant,
                modifier = Modifier.weight(weight = 1f),
            )
        }
    }
}
