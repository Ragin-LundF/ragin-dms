package dms.ui.shell

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.session.model.LibraryStatus
import dms.ui.session.model.SessionState
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/** The running operation with a spinner, or the state of the library when idle. */
@Composable
fun StatusFooter(sessionState: SessionState) {
    val colors = MaterialTheme.colorScheme
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.medium,
        color = colors.surfaceContainer,
    ) {
        Row(
            modifier = Modifier.padding(all = Spacing.medium),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            val busy = sessionState.operations.firstOrNull()?.label
            val idleText = when (sessionState.status) {
                LibraryStatus.Loading -> "Opening"
                LibraryStatus.NotConfigured -> "No library yet"
                is LibraryStatus.Open -> "Library ready"
                is LibraryStatus.PrimaryFailed, is LibraryStatus.Failed -> "Library not open"
            }
            if (busy != null) {
                CircularProgressIndicator(modifier = Modifier.size(size = 16.dp), strokeWidth = 2.dp)
            } else {
                val ready = sessionState.status is LibraryStatus.Open
                Box(
                    modifier = Modifier
                        .size(size = 10.dp)
                        .background(
                            color = if (ready) AppColors.statusGood else colors.error,
                            shape = CircleShape,
                        ),
                )
            }
            Column {
                Text(
                    text = if (busy != null) "Working" else idleText,
                    style = MaterialTheme.typography.labelLarge,
                )
                Text(
                    text = busy?.let { "$it…" } ?: "${sessionState.folders?.folders.orEmpty().size} library folders",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

