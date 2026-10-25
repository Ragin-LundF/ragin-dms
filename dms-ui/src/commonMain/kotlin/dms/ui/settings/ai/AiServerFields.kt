package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.width
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.application.port.state.model.AiSettingsModel
import dms.ui.settings.ai.model.AiDraftModel
import dms.ui.theme.Spacing

/**
 * The address of the server, the model to use, how long one answer may take, and the embedding
 * model for "Ask your library".
 */
@Composable
fun AiServerFields(draft: AiDraftModel, onChange: (AiDraftModel) -> Unit) {
    OutlinedTextField(
        value = draft.baseUrl,
        onValueChange = { onChange(draft.copy(baseUrl = it)) },
        label = { Text(text = "Server address") },
        supportingText = { Text(text = "LM Studio: http://localhost:1234/v1") },
        isError = draft.toSettings() == null,
        singleLine = true,
        modifier = Modifier.width(width = 520.dp),
    )
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        OutlinedTextField(
            value = draft.model,
            onValueChange = { onChange(draft.copy(model = it)) },
            label = { Text(text = "Model") },
            placeholder = { Text(text = "First loaded model") },
            singleLine = true,
            modifier = Modifier.width(width = 360.dp),
        )
        OutlinedTextField(
            value = draft.timeoutSeconds,
            onValueChange = { typed -> onChange(draft.copy(timeoutSeconds = typed.filter { it.isDigit() })) },
            label = { Text(text = "Timeout (seconds)") },
            singleLine = true,
            modifier = Modifier.width(width = 150.dp),
        )
    }
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        OutlinedTextField(
            value = draft.embeddingModel,
            onValueChange = { onChange(draft.copy(embeddingModel = it)) },
            label = { Text(text = "Embedding model for Ask") },
            placeholder = { Text(text = "First embedding model of the server") },
            singleLine = true,
            modifier = Modifier.width(width = 360.dp),
        )
        val range = AiSettingsModel.INPUT_CHARACTERS_RANGE
        OutlinedTextField(
            value = draft.inputCharacters,
            onValueChange = { typed -> onChange(draft.copy(inputCharacters = typed.filter { it.isDigit() })) },
            label = { Text(text = "Text sent to the AI (characters)") },
            supportingText = { Text(text = "${range.first} to ${range.last}; less for small models") },
            singleLine = true,
            modifier = Modifier.width(width = 260.dp),
        )
    }
}
