package dms.ui.shell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.KeyboardDoubleArrowLeft
import androidx.compose.material.icons.rounded.KeyboardDoubleArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import dms.ui.components.ReasonTooltip
import dms.ui.theme.Spacing

/** Switches the navigation between icons only and the full view with labels. */
@Composable
fun NavCollapseButton(collapsed: Boolean, onToggle: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val label = if (collapsed) "Expand menu" else "Collapse menu"
    val icon = if (collapsed) Icons.Rounded.KeyboardDoubleArrowRight else Icons.Rounded.KeyboardDoubleArrowLeft
    ReasonTooltip(reason = label.takeIf { collapsed }, beside = true) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape = MaterialTheme.shapes.medium)
                .clickable(onClickLabel = label, onClick = onToggle)
                .padding(all = Spacing.medium),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(
                space = Spacing.medium,
                alignment = Alignment.CenterHorizontally,
            ),
        ) {
            Icon(
                imageVector = icon,
                contentDescription = label,
                tint = colors.onSurfaceVariant,
                modifier = Modifier.size(size = Spacing.icon),
            )
            if (!collapsed) {
                Text(
                    text = label,
                    style = MaterialTheme.typography.labelLarge,
                    color = colors.onSurfaceVariant,
                    modifier = Modifier.weight(weight = 1f),
                )
            }
        }
    }
}
