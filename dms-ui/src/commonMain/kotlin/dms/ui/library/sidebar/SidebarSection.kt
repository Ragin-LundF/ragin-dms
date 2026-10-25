package dms.ui.library.sidebar

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.KeyboardArrowRight
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.KeyboardArrowDown
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.semantics
import dms.ui.components.ReasonTooltip
import dms.ui.theme.Spacing

/**
 * The heading of a sidebar section; a click on it, or on its chevron, folds the section. An optional
 * "+" adds an entry. In the [folded] rail the heading is a divider.
 */
@Composable
fun SidebarSection(
    title: String,
    expanded: Boolean,
    folded: Boolean,
    onToggle: () -> Unit,
    addLabel: String? = null,
    busyReason: String? = null,
    onAdd: () -> Unit = {},
) {
    if (folded) {
        HorizontalDivider(
            color = MaterialTheme.colorScheme.outlineVariant,
            modifier = Modifier.padding(vertical = Spacing.small),
        )
        return
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = Spacing.medium)
            .clip(shape = MaterialTheme.shapes.small)
            .clickable(
                role = Role.Button,
                onClickLabel = if (expanded) "Fold $title" else "Unfold $title",
                onClick = onToggle,
            )
            .semantics { stateDescription = if (expanded) "Unfolded" else "Folded" }
            .padding(start = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
    ) {
        Text(
            text = title.uppercase(),
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(weight = 1f),
        )
        if (addLabel != null) {
            ReasonTooltip(reason = busyReason ?: addLabel) {
                IconButton(
                    onClick = onAdd,
                    enabled = busyReason == null,
                    modifier = Modifier.size(size = Spacing.xlarge),
                ) {
                    Icon(
                        imageVector = Icons.Rounded.Add,
                        contentDescription = addLabel,
                        modifier = Modifier.size(size = Spacing.large),
                    )
                }
            }
        }
        Icon(
            imageVector = if (expanded) {
                Icons.Rounded.KeyboardArrowDown
            } else {
                Icons.AutoMirrored.Rounded.KeyboardArrowRight
            },
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.size(size = Spacing.large),
        )
    }
}
