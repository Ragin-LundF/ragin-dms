package dms.ui.document.details

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AddLink
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.domain.document.model.DocumentIdModel
import dms.ui.document.details.model.LinkPickModel
import dms.ui.theme.Spacing

/** "Link to…": search a document by name, title or correspondent, give the link a label, click it. */
@Composable
fun LinkPickerDialog(
    pick: LinkPickModel,
    onSearch: (String) -> Unit,
    onLabel: (String) -> Unit,
    onPick: (DocumentIdModel) -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 560.dp),
        icon = { Icon(imageVector = Icons.Rounded.AddLink, contentDescription = null) },
        title = { Text(text = "Link to another document") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
                OutlinedTextField(
                    value = pick.query,
                    onValueChange = onSearch,
                    label = { Text(text = "Search") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = pick.label,
                    onValueChange = onLabel,
                    label = { Text(text = "Label, such as \"reminder\" (optional)") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                LazyColumn(modifier = Modifier.heightIn(max = 320.dp).fillMaxWidth()) {
                    items(items = pick.matches, key = { it.documentId.value }) { candidate ->
                        Column(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { onPick(candidate.documentId) }
                                .padding(all = Spacing.small),
                        ) {
                            Text(text = candidate.name, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text(
                                text = candidate.path,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
            }
        },
        confirmButton = {},
        dismissButton = { TextButton(onClick = onDismiss) { Text(text = "Cancel") } },
    )
}
