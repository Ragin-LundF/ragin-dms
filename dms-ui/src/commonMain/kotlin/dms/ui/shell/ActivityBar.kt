package dms.ui.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Terminal
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.session.model.OperationProgress
import dms.ui.theme.Spacing

/**
 * The bar along the bottom of the window: what is running, how far it is and the file it is on, a
 * button to stop it when that is safe, and the switch for the log panel.
 */
@Composable
fun ActivityBar(
    busy: String?,
    progress: OperationProgress?,
    cancellable: Boolean,
    logVisible: Boolean,
    onCancel: () -> Unit,
    onToggleLog: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Surface(color = colors.surfaceContainerLow, modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.padding(horizontal = Spacing.large, vertical = Spacing.small),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            Column(
                modifier = Modifier.weight(weight = 1f),
                verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            ) {
                if (busy == null) {
                    Text(text = "Ready", style = MaterialTheme.typography.labelLarge, color = colors.onSurfaceVariant)
                } else {
                    ProgressLine(busy = busy, progress = progress)
                }
            }
            if (busy != null && cancellable) {
                TextButton(onClick = onCancel) {
                    Icon(
                        imageVector = Icons.Rounded.Close,
                        contentDescription = null,
                        modifier = Modifier.size(size = Spacing.icon),
                    )
                    Text(text = "Cancel", modifier = Modifier.padding(start = Spacing.xsmall))
                }
            }
            FilterChip(
                selected = logVisible,
                onClick = onToggleLog,
                label = { Text(text = "Log") },
                leadingIcon = {
                    Icon(
                        imageVector = Icons.Rounded.Terminal,
                        contentDescription = null,
                        modifier = Modifier.size(size = Spacing.icon),
                    )
                },
            )
        }
    }
}
