package dms.ui.settings.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.PlaylistAdd
import androidx.compose.material.icons.rounded.TextFields
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.unit.dp
import dms.domain.field.FieldRole
import dms.domain.field.FieldType
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.Spacing

/**
 * The library's custom fields: each with its type and, for dates, the deadline it stands for; a new
 * one by label, type and role; and the currency money values take when they name none.
 */
@Composable
fun FieldsCard(settings: CatalogSettingsModel, busyReason: String?, controller: LibrarySettingsController) {
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Custom fields",
            icon = Icons.Rounded.TextFields,
            tint = MaterialTheme.colorScheme.secondary,
            subtitle = "Values such as amounts and deadlines, on every document, for search and the dashboard",
        )
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            settings.fieldDefinitions.forEach { definition ->
                DefinitionRow(definition = definition, enabled = busyReason == null) {
                    controller.removeField(key = definition.key)
                }
            }
            NewFieldRow(busyReason = busyReason, controller = controller)
            CurrencyRow(
                current = settings.layout.defaultCurrency?.value.orEmpty(),
                busyReason = busyReason,
                controller = controller,
            )
            EmbedRow(embed = settings.layout.embedMetadata, busyReason = busyReason, controller = controller)
        }
    }
}

@Composable
private fun DefinitionRow(definition: FieldDefinitionModel, enabled: Boolean, onRemove: () -> Unit) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        val role = definition.role?.let { " · deadline: ${roleName(role = it)}" }.orEmpty()
        Text(
            text = "${definition.label}  (${definition.key}, ${typeName(type = definition.type)}$role)",
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.weight(weight = 1f),
        )
        IconButton(onClick = onRemove, enabled = enabled) {
            Icon(imageVector = Icons.Rounded.Delete, contentDescription = "Remove ${definition.label}")
        }
    }
}

@Composable
private fun NewFieldRow(busyReason: String?, controller: LibrarySettingsController) {
    var label by remember { mutableStateOf(value = "") }
    var type by remember { mutableStateOf(value = FieldType.TEXT) }
    var role by remember { mutableStateOf<FieldRole?>(value = null) }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        OutlinedTextField(
            value = label,
            onValueChange = { label = it },
            label = { Text(text = "New field") },
            singleLine = true,
            modifier = Modifier.weight(weight = 1f),
        )
        Choice(label = typeName(type = type), options = FieldType.entries, name = ::typeName) { type = it }
        if (type == FieldType.DATE) {
            Choice(
                label = role?.let { roleName(role = it) } ?: "No deadline",
                options = listOf<FieldRole?>(null) + FieldRole.entries,
                name = { it?.let { value -> roleName(role = value) } ?: "No deadline" },
            ) { role = it }
        }
    }
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(
                label = "Add field",
                icon = Icons.Rounded.Add,
                disabledReason = "Name the field".takeIf { label.isBlank() },
            ) {
                controller.addField(label = label, type = type, role = role)
                label = ""
            },
            ActionButtonModel(
                label = "Add standard fields",
                icon = Icons.Rounded.PlaylistAdd,
                onClick = controller::addStandardFields,
            ),
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}

@Composable
private fun CurrencyRow(current: String, busyReason: String?, controller: LibrarySettingsController) {
    var typed by remember(current) { mutableStateOf(value = current) }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        OutlinedTextField(
            value = typed,
            onValueChange = { typed = it },
            label = { Text(text = "Default currency, such as EUR") },
            singleLine = true,
            modifier = Modifier.weight(weight = 1f),
        )
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(
                    label = "Save currency",
                    disabledReason = "Nothing changed".takeIf { typed.trim() == current },
                ) {
                    controller.saveCurrency(typed = typed)
                },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}

@Composable
private fun <T> Choice(label: String, options: List<T>, name: (T) -> String, onPick: (T) -> Unit) {
    var open by remember { mutableStateOf(value = false) }
    Box {
        OutlinedButton(onClick = { open = true }) {
            Text(text = label, maxLines = 1, softWrap = false)
            Icon(imageVector = Icons.Rounded.ArrowDropDown, contentDescription = null)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            options.forEach { option ->
                DropdownMenuItem(
                    text = { Text(text = name(option)) },
                    onClick = {
                        onPick(option)
                        open = false
                    },
                )
            }
        }
    }
}

@Composable
private fun EmbedRow(embed: Boolean, busyReason: String?, controller: LibrarySettingsController) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(
            text = "Also write approved title, correspondent, category, tags and date into PDF files",
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.weight(weight = 1f),
        )
        Switch(checked = embed, onCheckedChange = controller::setEmbedMetadata, enabled = busyReason == null)
    }
}

private fun typeName(type: FieldType): String {
    return when (type) {
        FieldType.TEXT -> "Text"
        FieldType.NUMBER -> "Number"
        FieldType.MONEY -> "Amount"
        FieldType.DATE -> "Date"
        FieldType.BOOLEAN -> "Yes/no"
    }
}

private fun roleName(role: FieldRole): String {
    return when (role) {
        FieldRole.DUE_DATE -> "Due date"
        FieldRole.CANCEL_BY -> "Cancel by"
        FieldRole.CONTRACT_END -> "Contract end"
        FieldRole.WARRANTY_END -> "Warranty end"
    }
}
