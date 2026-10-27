package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import dms.ui.settings.ai.model.AiDraftModel
import dms.ui.theme.Spacing

/**
 * How long one answer may take and how much text is sent, then where the AI runs, Local AI or Anthropic,
 * with the fields of that choice. The shared fields come first, so they keep their place on a switch.
 */
@Composable
fun AiServerFields(draft: AiDraftModel, onChange: (AiDraftModel) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        OutlinedTextField(
            value = draft.timeoutSeconds,
            onValueChange = { typed -> onChange(draft.copy(timeoutSeconds = typed.filter { it.isDigit() })) },
            label = { Text(text = "Timeout (seconds)") },
            isError = !draft.timeoutValid(),
            singleLine = true,
            modifier = Modifier.width(width = 150.dp),
        )
        val range = AiSettingsModel.INPUT_CHARACTERS_RANGE
        OutlinedTextField(
            value = draft.inputCharacters,
            onValueChange = { typed -> onChange(draft.copy(inputCharacters = typed.filter { it.isDigit() })) },
            label = { Text(text = "Text sent to the AI (characters)") },
            supportingText = { Text(text = "${range.first} to ${range.last}; less for small models") },
            isError = !draft.inputValid(),
            singleLine = true,
            modifier = Modifier.width(width = 260.dp),
        )
    }
    AiProviderChips(selected = draft.provider, onSelect = { onChange(draft.copy(provider = it)) })
    when (draft.provider) {
        AiProvider.LOCAL -> LocalAiFields(draft = draft, onChange = onChange)
        AiProvider.ANTHROPIC -> AnthropicFields(draft = draft, onChange = onChange)
    }
}

/** An OpenAI-compatible server such as LM Studio: its address, the model and the embedding model for Ask. */
@Composable
private fun LocalAiFields(draft: AiDraftModel, onChange: (AiDraftModel) -> Unit) {
    OutlinedTextField(
        value = draft.baseUrl,
        onValueChange = { onChange(draft.copy(baseUrl = it)) },
        label = { Text(text = "Server address") },
        supportingText = { Text(text = "LM Studio: http://localhost:1234/v1") },
        isError = !draft.addressValid(),
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
            value = draft.embeddingModel,
            onValueChange = { onChange(draft.copy(embeddingModel = it)) },
            label = { Text(text = "Embedding model for Ask") },
            placeholder = { Text(text = "First embedding model of the server") },
            singleLine = true,
            modifier = Modifier.width(width = 360.dp),
        )
    }
}

/** Anthropic's API: only the Claude model; the address is fixed and the key comes from the environment. */
@Composable
private fun AnthropicFields(draft: AiDraftModel, onChange: (AiDraftModel) -> Unit) {
    OutlinedTextField(
        value = draft.anthropicModel,
        onValueChange = { onChange(draft.copy(anthropicModel = it)) },
        label = { Text(text = "Claude model") },
        placeholder = { Text(text = AiSettingsModel.DEFAULT_ANTHROPIC_MODEL) },
        singleLine = true,
        modifier = Modifier.width(width = 360.dp),
    )
    Text(
        text = "The API key is read from ANTHROPIC_API_KEY when the app starts. " +
            "Ask still uses the embedding model of Local AI.",
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        // Lines up with the fields' own supporting text.
        modifier = Modifier.padding(start = Spacing.large),
    )
}
