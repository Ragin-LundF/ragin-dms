package dms.ui.inbox

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.DriveFileMove
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
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
import dms.ui.inbox.model.FolderPickModel
import dms.ui.theme.Spacing

/**
 * "Move to…": a folder of the library, searched by typing, or a new path, which is created; recently
 * used folders come first. The file name can be changed on the way.
 */
@Composable
fun FolderPickerDialog(
    pick: FolderPickModel,
    folders: List<String>,
    recent: List<String>,
    onChange: (folder: String, fileName: String) -> Unit,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    val typed = pick.folder.trim().trim('/')
    AlertDialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 560.dp),
        icon = { Icon(imageVector = Icons.AutoMirrored.Rounded.DriveFileMove, contentDescription = null) },
        title = { Text(text = "Move ${pick.documentName}") },
        text = { PickerContent(pick = pick, folders = folders, recent = recent, onChange = onChange) },
        confirmButton = {
            val ready = typed.isNotEmpty() && pick.fileName.isNotBlank()
            Button(onClick = onConfirm, enabled = ready) { Text(text = "Move") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text(text = "Cancel") } },
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun PickerContent(
    pick: FolderPickModel,
    folders: List<String>,
    recent: List<String>,
    onChange: (folder: String, fileName: String) -> Unit,
) {
    val typed = pick.folder.trim().trim('/')
    val matches = folders.filter { it.contains(other = typed, ignoreCase = true) }
    val isNew = typed.isNotEmpty() && folders.none { it.equals(other = typed, ignoreCase = true) }
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        if (recent.isNotEmpty()) {
            Text(text = "Recently used", style = MaterialTheme.typography.labelLarge)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
                recent.forEach { folder ->
                    AssistChip(
                        onClick = { onChange(folder, pick.fileName) },
                        label = { Text(text = folder, maxLines = 1, softWrap = false) },
                    )
                }
            }
        }
        OutlinedTextField(
            value = pick.folder,
            onValueChange = { onChange(it, pick.fileName) },
            label = { Text(text = "Folder: search, pick, or type a new path") },
            singleLine = true,
            leadingIcon = {
                Icon(
                    imageVector = if (isNew) Icons.Rounded.CreateNewFolder else Icons.Rounded.Folder,
                    contentDescription = null,
                )
            },
            supportingText = { if (isNew) Text(text = "\"$typed\" will be created in every library folder") },
            modifier = Modifier.fillMaxWidth(),
        )
        LazyColumn(modifier = Modifier.heightIn(max = 240.dp).fillMaxWidth()) {
            items(items = matches) { folder ->
                Text(
                    text = folder,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { onChange(folder, pick.fileName) }
                        .padding(vertical = Spacing.small, horizontal = Spacing.small),
                )
            }
        }
        OutlinedTextField(
            value = pick.fileName,
            onValueChange = { onChange(pick.folder, it) },
            label = { Text(text = "File name") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
