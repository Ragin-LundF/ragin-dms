package dms.ui.shell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Terminal
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.OperationLane
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.SessionStateModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing
import java.time.Instant
import java.time.ZoneId

/**
 * The one status bar along the bottom of the window: on the left the state of the library and the
 * last scan, on the right every running operation with its progress, Cancel where stopping is safe,
 * and the switch for the log panel.
 */
@Composable
fun ActivityBar(
    sessionState: SessionStateModel,
    progress: Map<OperationLane, OperationProgressModel>,
    logVisible: Boolean,
    onCancel: (OperationLane) -> Unit,
    onToggleLog: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Surface(color = colors.surfaceContainerLow, modifier = Modifier.fillMaxWidth()) {
        HorizontalDivider(color = colors.outlineVariant)
        Row(
            modifier = Modifier.height(height = Spacing.statusBarHeight).padding(horizontal = Spacing.large),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            Box(
                modifier = Modifier
                    .size(size = Spacing.small)
                    .background(color = statusColor(status = sessionState.status), shape = CircleShape),
            )
            SmallText(
                text = StatusTexts.library(state = sessionState, now = Instant.now(), zone = ZoneId.systemDefault()),
                modifier = Modifier.weight(weight = 1f),
            )
            sessionState.operations.forEach { operation ->
                ProgressLine(busy = operation.label, progress = progress[operation.lane])
                if (operation.cancellable) {
                    TextButton(onClick = { onCancel(operation.lane) }) {
                        Icon(
                            imageVector = Icons.Rounded.Close,
                            contentDescription = null,
                            modifier = Modifier.size(size = Spacing.medium),
                        )
                        SmallText(text = "Cancel", modifier = Modifier.padding(start = Spacing.xsmall))
                    }
                }
            }
            TextButton(onClick = onToggleLog) {
                Icon(
                    imageVector = Icons.Rounded.Terminal,
                    contentDescription = null,
                    tint = if (logVisible) colors.primary else colors.onSurfaceVariant,
                    modifier = Modifier.size(size = Spacing.medium),
                )
                SmallText(
                    text = if (logVisible) "Hide log" else "Log",
                    modifier = Modifier.padding(start = Spacing.xsmall),
                )
            }
        }
    }
}

@Composable
private fun SmallText(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelSmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier,
    )
}

@Composable
private fun statusColor(status: LibraryStatus): Color {
    return when (status) {
        is LibraryStatus.Open -> AppColors.statusGood
        LibraryStatus.Loading -> AppColors.statusWarning
        LibraryStatus.NotConfigured,
        is LibraryStatus.PrimaryFailed,
        is LibraryStatus.Failed,
        -> MaterialTheme.colorScheme.error
    }
}
