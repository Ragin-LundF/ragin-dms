package dms.ui.restructure

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AccountTree
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material.icons.rounded.Preview
import androidx.compose.material.icons.rounded.Save
import androidx.compose.material3.AssistChip
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.restructure.FileNaming
import dms.domain.restructure.template.PlaceholderName
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.restructure.model.RestructureUiStateModel
import dms.ui.session.OutcomeMessages
import dms.ui.theme.Spacing

/**
 * What to restructure and how: the scope, the path template with a chip per placeholder, the file
 * naming, and the buttons to preview the plan and to keep the template as the library's default.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TemplateCard(state: RestructureUiStateModel, viewModel: RestructureViewModel) {
    AppCard(modifier = Modifier.fillMaxWidth()) {
        CardTitle(
            title = "Template",
            icon = Icons.Rounded.AccountTree,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = "Folders separated by /, values in braces, a fallback after |, such as {year|Undated}",
        )
        ScopePicker(state = state, onSelect = viewModel::selectScope)
        OutlinedTextField(
            value = state.template,
            onValueChange = viewModel::setTemplate,
            label = { Text(text = "Path template") },
            singleLine = true,
            isError = state.problem != null,
            supportingText = { state.problem?.let { Text(text = it) } },
            modifier = Modifier.fillMaxWidth(),
        )
        FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            PlaceholderName.entries.forEach { placeholder ->
                val token = if (placeholder == PlaceholderName.FIELD) "field:due" else placeholder.token
                AssistChip(
                    onClick = { viewModel.insertPlaceholder(token = token) },
                    label = { Text(text = "{$token}", maxLines = 1, softWrap = false) },
                )
            }
        }
        NamingRow(naming = state.naming, onChange = viewModel::setNaming)
        TemplateButtons(state = state, viewModel = viewModel)
    }
}

@Composable
private fun ScopePicker(state: RestructureUiStateModel, onSelect: (Int) -> Unit) {
    var open by remember { mutableStateOf(value = false) }
    Box {
        OutlinedButton(onClick = { open = true }) {
            Text(
                text = "Scope: ${state.scopes.getOrNull(index = state.scopeIndex)?.label.orEmpty()}",
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Icon(imageVector = Icons.Rounded.ArrowDropDown, contentDescription = null)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            state.scopes.forEachIndexed { index, option ->
                DropdownMenuItem(
                    text = { Text(text = option.label, maxLines = 1, overflow = TextOverflow.Ellipsis) },
                    onClick = {
                        onSelect(index)
                        open = false
                    },
                )
            }
        }
    }
}

@Composable
private fun NamingRow(naming: FileNaming, onChange: (FileNaming) -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Switch(
            checked = naming == FileNaming.DATE_AND_TITLE,
            onCheckedChange = { renamed -> onChange(if (renamed) FileNaming.DATE_AND_TITLE else FileNaming.KEEP) },
        )
        Text(text = "Rename to \"date title\" where a document has both", style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun TemplateButtons(state: RestructureUiStateModel, viewModel: RestructureViewModel) {
    val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(
                label = "Preview",
                icon = Icons.Rounded.Preview,
                style = ButtonStyle.TONAL,
                onClick = viewModel::preview,
            ),
            ActionButtonModel(
                label = "Save as default",
                icon = Icons.Rounded.Save,
                disabledReason = "This is the saved template".takeIf { state.savedTemplate == state.template.trim() },
                onClick = viewModel::saveTemplate,
            ),
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
