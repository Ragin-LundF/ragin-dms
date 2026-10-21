package dms.ui.library.sidebar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dms.ui.components.FoldToggle
import dms.ui.components.ReasonTooltip
import dms.ui.theme.Spacing

/**
 * The heading of a sidebar section: a boxed +/- folds the section, an optional "+" adds an entry.
 * [highlighted] tints the heading while something dragged hovers over it.
 */
@Composable
fun SidebarSection(
    title: String,
    expanded: Boolean,
    onToggle: () -> Unit,
    addLabel: String? = null,
    busyReason: String? = null,
    highlighted: Boolean = false,
    onAdd: () -> Unit = {},
) {
    val tint = if (highlighted) MaterialTheme.colorScheme.primary.copy(alpha = 0.16f) else Color.Transparent
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = Spacing.large)
            .background(color = tint, shape = RoundedCornerShape(size = 8.dp))
            .padding(start = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        FoldToggle(expanded = expanded, onToggle = onToggle)
        Text(
            text = title.uppercase(),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.clickable(onClick = onToggle),
        )
        Spacer(modifier = Modifier.weight(weight = 1f))
        if (addLabel != null) {
            ReasonTooltip(reason = busyReason ?: addLabel) {
                IconButton(onClick = onAdd, enabled = busyReason == null, modifier = Modifier.size(size = 28.dp)) {
                    Icon(imageVector = Icons.Rounded.Add, contentDescription = addLabel)
                }
            }
        }
    }
}
