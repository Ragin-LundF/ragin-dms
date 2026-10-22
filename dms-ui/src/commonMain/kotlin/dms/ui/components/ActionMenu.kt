package dms.ui.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.model.ActionButton
import dms.ui.theme.Spacing

/**
 * A 3-dot button that opens [actions] as a menu; a disabled action explains in its tooltip why it
 * cannot be used.
 */
@Composable
fun ActionMenu(
    actions: List<ActionButton>,
    contentDescription: String,
    enabled: Boolean,
    busyReason: String?,
) {
    var open by remember { mutableStateOf(value = false) }
    Box {
        IconButton(onClick = { open = true }, modifier = Modifier.size(size = 24.dp)) {
            Icon(
                imageVector = Icons.Rounded.MoreVert,
                contentDescription = contentDescription,
                modifier = Modifier.size(size = Spacing.icon),
            )
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            actions.forEach { action ->
                val reason = action.disabledReason ?: busyReason.takeUnless { enabled }
                ReasonTooltip(reason = reason) {
                    DropdownMenuItem(
                        text = { Text(text = action.label) },
                        leadingIcon = action.icon?.let { icon ->
                            { Icon(imageVector = icon, contentDescription = null) }
                        },
                        enabled = reason == null,
                        onClick = {
                            open = false
                            action.onClick()
                        },
                    )
                }
            }
        }
    }
}
