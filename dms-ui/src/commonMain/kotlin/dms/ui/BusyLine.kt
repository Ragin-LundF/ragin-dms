package dms.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.theme.Spacing

/** The running operation, if any; keeps its height when idle so the screen does not jump. */
@Composable
fun BusyLine(busy: String?) {
    Column(modifier = Modifier.fillMaxWidth().height(height = 28.dp).padding(horizontal = Spacing.large)) {
        if (busy != null) {
            LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            Text(text = "$busy…", style = MaterialTheme.typography.bodySmall, maxLines = 1, softWrap = false)
        }
    }
}
