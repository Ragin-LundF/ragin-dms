package dms.ui.document.details

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.PlaylistAdd
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentModel
import dms.domain.field.FieldType
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.theme.Spacing

/**
 * The custom fields of the library, one typed editor per field, saved together. A library without
 * fields offers the standard ones.
 */
@Composable
fun FieldsEditor(
    document: DocumentModel,
    definitions: List<FieldDefinitionModel>,
    busyReason: String?,
    onSave: (Map<FieldKeyModel, String>) -> Unit,
    onAddStandardFields: () -> Unit,
) {
    if (definitions.isEmpty()) {
        Text(
            text = "No custom fields yet. Add the standard ones (amount, due date, cancel by, contract end, " +
                "warranty) or define your own in Settings.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(
                    label = "Add standard fields",
                    icon = Icons.Rounded.PlaylistAdd,
                    onClick = onAddStandardFields,
                ),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
        return
    }
    val saved = definitions.associate { definition ->
        definition.key to document.fields[definition.key]?.let { FieldValues.format(value = it) }.orEmpty()
    }
    var typed by remember(key1 = document.id, key2 = saved) { mutableStateOf(value = saved) }
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        definitions.forEach { definition ->
            OutlinedTextField(
                value = typed[definition.key].orEmpty(),
                onValueChange = { typed = typed + (definition.key to it) },
                label = { Text(text = "${definition.label} (${hintOf(type = definition.type)})") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
        }
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(
                    label = "Save fields",
                    icon = Icons.Rounded.Check,
                    style = ButtonStyle.TONAL,
                    disabledReason = "Nothing changed".takeIf { typed == saved },
                ) { onSave(typed) },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}

private fun hintOf(type: FieldType): String {
    return when (type) {
        FieldType.TEXT -> "text"
        FieldType.NUMBER -> "number"
        FieldType.MONEY -> "amount and currency"
        FieldType.DATE -> "YYYY-MM-DD"
        FieldType.BOOLEAN -> "yes or no"
    }
}
