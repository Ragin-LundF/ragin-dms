package dms.ui.library.sidebar.item

import androidx.compose.foundation.background
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ReasonTooltip
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * One view of the sidebar with its icon and count. A [warning] count is a badge in the warning tint.
 * In the [folded] rail only the icon shows, the badge becomes a dot and a tooltip names the view.
 */
@Composable
fun SidebarItem(
    label: String,
    icon: ImageVector,
    count: Int?,
    selected: Boolean,
    folded: Boolean,
    warning: Boolean = false,
    tooltip: String? = null,
    menu: (@Composable () -> Unit)? = null,
    onClick: () -> Unit,
) {
    val clickable = Modifier.clickable(onClick = onClick)
    ReasonTooltip(reason = tooltip ?: label.takeIf { folded }, beside = true) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape = MaterialTheme.shapes.small)
                .background(color = if (selected) MaterialTheme.colorScheme.surface else Color.Transparent)
                .then(other = clickable)
                .semantics { if (folded) contentDescription = label }
                .padding(
                    horizontal = if (folded) Spacing.xsmall else Spacing.small,
                    vertical = Spacing.xsmall + Spacing.hairline * 2,
                ),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = if (folded) Arrangement.Center else Arrangement.spacedBy(space = Spacing.small),
        ) {
            ItemIcon(icon = icon, selected = selected, dot = folded && warning && (count ?: 0) > 0)
            if (!folded) {
                ItemLabel(label = label, selected = selected, modifier = Modifier.weight(weight = 1f))
                count?.let { Count(count = it, warning = warning) }
                menu?.invoke()
            }
        }
    }
}

@Composable
private fun ItemLabel(label: String, selected: Boolean, modifier: Modifier) {
    Text(
        text = label,
        style = MaterialTheme.typography.bodyMedium,
        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium,
        color = if (selected) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        softWrap = false,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier,
    )
}

@Composable
private fun ItemIcon(icon: ImageVector, selected: Boolean, dot: Boolean) {
    Box {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(size = Spacing.icon),
        )
        if (dot) {
            Box(
                modifier = Modifier
                    .align(alignment = Alignment.TopEnd)
                    .size(size = Spacing.small)
                    .background(color = AppColors.statusWarning, shape = CircleShape),
            )
        }
    }
}

@Composable
private fun Count(count: Int, warning: Boolean) {
    val badge = warning && count > 0
    Text(
        text = count.toString(),
        style = MaterialTheme.typography.labelSmall,
        fontWeight = if (badge) FontWeight.SemiBold else FontWeight.Normal,
        color = if (badge) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        softWrap = false,
        modifier = if (badge) {
            Modifier
                .background(color = AppColors.statusWarning.copy(alpha = 0.28f), shape = CircleShape)
                .padding(horizontal = Spacing.xsmall + Spacing.hairline * 2)
        } else {
            Modifier
        },
    )
}
