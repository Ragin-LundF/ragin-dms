package dms.ui.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.theme.Spacing

/** The title, the filter field and the actions that add to the selected folder. */
@Composable
fun LibraryToolbar(
    query: String,
    busyReason: String?,
    onSearch: (String) -> Unit,
    onAddFiles: () -> Unit,
    onNewFolder: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(bottom = Spacing.medium),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Text(text = "Library", style = MaterialTheme.typography.headlineSmall)
        OutlinedTextField(
            value = query,
            onValueChange = onSearch,
            placeholder = { Text(text = "Filter by name, title, correspondent or tag") },
            singleLine = true,
            modifier = Modifier.weight(weight = 1f).width(width = 320.dp),
        )
        ActionButtons(
            buttons = listOf(
                ActionButton(label = "Add files…", onClick = onAddFiles),
                ActionButton(label = "New folder…", onClick = onNewFolder),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
