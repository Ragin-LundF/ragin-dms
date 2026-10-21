package dms.ui.library.sidebar

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ReasonTooltip
import dms.ui.theme.Spacing

/** The heading of a sidebar section, with an optional "+" that adds an entry. */
@Composable
fun SidebarSection(title: String, addLabel: String? = null, busyReason: String? = null, onAdd: () -> Unit = {}) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(start = Spacing.small, top = Spacing.large),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title.uppercase(),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
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
