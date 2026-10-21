package dms.ui.dashboard

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.application.port.state.model.ActivityEntry
import dms.ui.components.SectionHeader

/** What the tool did recently on this device. */
@Composable
fun ActivityPanel(entries: List<ActivityEntry>) {
    Column {
        SectionHeader(title = "Recent activity")
        if (entries.isEmpty()) {
            Text(text = "Nothing yet", style = MaterialTheme.typography.bodySmall)
        }
        LazyColumn(modifier = Modifier.heightIn(max = 140.dp)) {
            items(items = entries) { entry ->
                Text(
                    text = ActivityTexts.line(entry = entry),
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}
