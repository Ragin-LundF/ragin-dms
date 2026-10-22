package dms.ui.export

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Archive
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.ui.export.model.ExportDraftModel
import dms.ui.theme.Spacing

/** The export [controller] is setting up: presets, name, layout, then the target folder. */
@Composable
fun ExportDialog(controller: ExportController) {
    val open by controller.draft.collectAsState()
    val draft = open ?: return
    AlertDialog(
        onDismissRequest = controller::cancel,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 600.dp),
        icon = { Icon(imageVector = Icons.Rounded.Archive, contentDescription = null) },
        title = { Text(text = "Export: ${draft.scopeLabel}") },
        text = { ExportContent(draft = draft, controller = controller) },
        confirmButton = {
            Button(onClick = controller::export, enabled = draft.name.isNotBlank()) {
                Text(text = "Choose folder and export")
            }
        },
        dismissButton = { TextButton(onClick = controller::cancel) { Text(text = "Cancel") } },
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ExportContent(draft: ExportDraftModel, controller: ExportController) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        Text(text = "Presets", style = MaterialTheme.typography.labelLarge)
        FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            controller.taxYears.forEach { year ->
                AssistChip(
                    onClick = { controller.presetTaxYear(year = year) },
                    label = { Text(text = "Tax year $year") },
                )
            }
        }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            OutlinedTextField(
                value = draft.correspondent,
                onValueChange = controller::setCorrespondent,
                label = { Text(text = "Everything for correspondent…") },
                singleLine = true,
                modifier = Modifier.weight(weight = 1f),
            )
            TextButton(onClick = controller::presetCorrespondent, enabled = draft.correspondent.isNotBlank()) {
                Text(text = "Use")
            }
        }
        OutlinedTextField(
            value = draft.name,
            onValueChange = controller::setName,
            label = { Text(text = "Name of the export folder or ZIP file") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        val options = draft.options
        OptionRow(text = "Keep the folder structure", checked = !options.flatten) {
            controller.setOptions(options = options.copy(flatten = !it))
        }
        OptionRow(text = "Name files after their title", checked = options.renameToTitle) {
            controller.setOptions(options = options.copy(renameToTitle = it))
        }
        OptionRow(text = "Add an index (index.csv and index.html)", checked = options.index) {
            controller.setOptions(options = options.copy(index = it))
        }
        OptionRow(text = "One ZIP file instead of a folder", checked = options.zip) {
            controller.setOptions(options = options.copy(zip = it))
        }
    }
}

@Composable
private fun OptionRow(text: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(text = text, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(weight = 1f))
        Switch(checked = checked, onCheckedChange = onChange)
    }
}
