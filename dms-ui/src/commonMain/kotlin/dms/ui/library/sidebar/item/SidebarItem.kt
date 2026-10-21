package dms.ui.library.sidebar.item

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.FoldToggle
import dms.ui.components.surface.CountPill
import dms.ui.library.sidebar.model.TreeToggle
import dms.ui.theme.Spacing

/**
 * One selectable line of the sidebar, indented by [depth]; it lights up while a document hovers over
 * it. A tree entry has a [toggle] that folds its children.
 */
@Composable
fun SidebarItem(
    label: String,
    icon: ImageVector,
    count: Int?,
    depth: Int,
    selected: Boolean,
    dropHovered: Boolean,
    toggle: TreeToggle? = null,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.small
    val background = when {
        dropHovered -> colors.primary.copy(alpha = 0.16f)
        selected -> colors.primaryContainer
        else -> Color.Transparent
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = shape)
            .background(color = background)
            .border(width = 2.dp, color = if (dropHovered) colors.primary else Color.Transparent, shape = shape)
            .clickable(onClick = onClick)
            .padding(start = Spacing.small + Spacing.large * depth, end = Spacing.small)
            .padding(vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        toggle?.let { FoldToggle(expanded = it.expanded, onToggle = it.onToggle) }
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = if (selected || dropHovered) colors.primary else colors.onSurfaceVariant,
            modifier = Modifier.size(size = Spacing.icon),
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodyMedium,
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
            color = if (selected) colors.onPrimaryContainer else colors.onSurface,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(weight = 1f),
        )
        count?.let { CountPill(count = it, selected = selected) }
    }
}
