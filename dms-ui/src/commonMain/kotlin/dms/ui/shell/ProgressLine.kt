package dms.ui.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.width
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.session.model.OperationProgressModel
import dms.ui.theme.Spacing
import java.util.Locale

/** A running operation in one short line for the status bar: its step, its counts and a small progress bar. */
@Composable
fun ProgressLine(busy: String, progress: OperationProgressModel?) {
    val colors = MaterialTheme.colorScheme
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        val detail = progress?.detail?.let { " · $it" }.orEmpty()
        Text(
            text = (progress?.step ?: busy) + detail,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.widthIn(max = Spacing.statusTextMaxWidth),
        )
        progress?.total?.let { total ->
            Text(
                text = String.format(Locale.getDefault(), "%,d of %,d", progress.done, total),
                style = MaterialTheme.typography.labelSmall,
                color = colors.primary,
                maxLines = 1,
                softWrap = false,
            )
        }
        val fraction = progress?.fraction
        val bar = Modifier.width(width = Spacing.statusProgressWidth)
        if (fraction == null) {
            LinearProgressIndicator(modifier = bar)
        } else {
            LinearProgressIndicator(progress = { fraction }, modifier = bar)
        }
    }
}
