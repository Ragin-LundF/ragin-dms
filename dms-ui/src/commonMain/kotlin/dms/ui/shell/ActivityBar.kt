package dms.ui.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import dms.ui.session.model.OperationLane
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.RunningOperationModel
import dms.ui.theme.Spacing

/**
 * The bar along the bottom of the window: what is running, one line per operation, how far each is
 * and the file it is on, a button to stop it when that is safe, and the switch for the log panel.
 */
@Composable
fun ActivityBar(
    operations: List<RunningOperationModel>,
    progress: Map<OperationLane, OperationProgressModel>,
    logVisible: Boolean,
    onCancel: (OperationLane) -> Unit,
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
                if (operations.isEmpty()) {
                    Text(text = "Ready", style = MaterialTheme.typography.labelLarge, color = colors.onSurfaceVariant)
                }
                operations.forEach { operation ->
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(modifier = Modifier.weight(weight = 1f)) {
                            ProgressLine(busy = operation.label, progress = progress[operation.lane])
                        }
                        if (operation.cancellable) {
                            TextButton(onClick = { onCancel(operation.lane) }) {
                                Icon(
                                    imageVector = Icons.Rounded.Close,
                                    contentDescription = null,
                                    modifier = Modifier.size(size = Spacing.icon),
                                )
                                Text(text = "Cancel", modifier = Modifier.padding(start = Spacing.xsmall))
                            }
                        }
                    }
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
