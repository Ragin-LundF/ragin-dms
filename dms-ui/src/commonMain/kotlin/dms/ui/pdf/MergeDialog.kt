package dms.ui.pdf

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.MergeType
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.ui.pdf.model.MergeStateModel
import dms.ui.theme.Spacing

/** "Merge with…": the PDFs in their order, a search to add more, the new name, and whether the sources go. */
@Composable
fun MergeDialog(actions: PdfActions) {
    val open by actions.merge.collectAsState()
    val state = open ?: return
    AlertDialog(
        onDismissRequest = actions::cancelMerge,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 600.dp),
        icon = { Icon(imageVector = Icons.Rounded.MergeType, contentDescription = null) },
        title = { Text(text = "Merge PDFs") },
        text = { MergeContent(state = state, actions = actions) },
        confirmButton = {
            Button(onClick = actions::confirmMerge, enabled = state.chosen.size >= 2 && state.name.isNotBlank()) {
                Text(text = "Merge ${state.chosen.size} PDFs")
            }
        },
        dismissButton = { TextButton(onClick = actions::cancelMerge) { Text(text = "Cancel") } },
    )
}

@Composable
private fun MergeContent(state: MergeStateModel, actions: PdfActions) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        state.chosen.forEachIndexed { index, candidate ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "${index + 1}. ${candidate.name}",
                    modifier = Modifier.weight(weight = 1f),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (index > 0) {
                    IconButton(onClick = { actions.remove(documentId = candidate.documentId) }) {
                        Icon(imageVector = Icons.Rounded.Close, contentDescription = "Leave out ${candidate.name}")
                    }
                }
            }
        }
        OutlinedTextField(
            value = state.query,
            onValueChange = actions::search,
            label = { Text(text = "Add a PDF: search") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        LazyColumn(modifier = Modifier.heightIn(max = 200.dp).fillMaxWidth()) {
            items(items = state.matches, key = { it.documentId.value }) { candidate ->
                Text(
                    text = candidate.path,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { actions.add(documentId = candidate.documentId) }
                        .padding(all = Spacing.small),
                )
            }
        }
        OutlinedTextField(
            value = state.name,
            onValueChange = actions::setName,
            label = { Text(text = "Name of the merged file") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(verticalAlignment = Alignment.CenterVertically) {
            Checkbox(checked = state.trashSources, onCheckedChange = actions::setTrashSources)
            Text(text = "Move the merged PDFs to the trash afterwards", style = MaterialTheme.typography.bodyMedium)
        }
    }
}
