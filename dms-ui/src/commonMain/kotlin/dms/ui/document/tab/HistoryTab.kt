package dms.ui.document.tab

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.format.DateTexts
import dms.ui.document.model.HistoryEntryModel
import dms.ui.theme.Spacing
import java.time.ZoneId

/** What happened to the document, newest first, and what the list leaves out. */
@Composable
fun HistoryTab(entries: List<HistoryEntryModel>) {
    val colors = MaterialTheme.colorScheme
    val zone = ZoneId.systemDefault()
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        entries.forEach { entry ->
            Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
                Text(
                    text = DateTexts.absolute(instant = entry.at, zone = zone),
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                )
                Text(
                    text = entry.text,
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.weight(weight = 1f),
                )
            }
        }
        Text(
            text = "Shows when the document was added, its last classification and this device's recent activity.",
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
        )
    }
}
