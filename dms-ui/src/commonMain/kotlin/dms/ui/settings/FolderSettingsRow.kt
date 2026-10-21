package dms.ui.settings

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.settings.model.FolderRow
import dms.ui.theme.Spacing

/** One library folder with its role and the actions it allows. */
@Composable
fun FolderSettingsRow(row: FolderRow, busyReason: String?, onMakePrimary: () -> Unit, onRemove: () -> Unit) {
    OutlinedCard(modifier = Modifier.fillMaxWidth()) {
        Row(modifier = Modifier.padding(all = Spacing.medium), verticalAlignment = Alignment.CenterVertically) {
            Column(modifier = Modifier.weight(weight = 1f)) {
                val role = if (row.isPrimary) "★ Primary" else "● Sync folder"
                Text(text = "$role · ${row.folder.displayName}", style = MaterialTheme.typography.titleSmall)
                Text(
                    text = row.folder.location.value,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
                row.note?.let { note ->
                    Text(
                        text = note,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }
            if (!row.isPrimary) {
                ActionButtons(
                    buttons = listOf(
                        ActionButton(label = "Make primary", onClick = onMakePrimary),
                        ActionButton(label = "Remove", onClick = onRemove),
                    ),
                    enabled = busyReason == null,
                    busyReason = busyReason,
                )
            }
        }
    }
}
