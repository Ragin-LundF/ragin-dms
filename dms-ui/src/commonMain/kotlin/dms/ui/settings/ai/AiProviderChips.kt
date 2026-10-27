package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.application.port.state.model.AiProvider
import dms.ui.theme.Spacing

/** Who answers: Local AI or Anthropic, one choice of two. Switching keeps the fields of both. */
@Composable
fun AiProviderChips(selected: AiProvider, onSelect: (AiProvider) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        Text(
            text = "Runs on",
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Row(
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            modifier = Modifier.selectableGroup(),
        ) {
            AiProvider.entries.forEach { provider ->
                FilterChip(
                    selected = selected == provider,
                    onClick = { onSelect(provider) },
                    label = { Text(text = labelOf(provider = provider), softWrap = false) },
                )
            }
        }
    }
}

private fun labelOf(provider: AiProvider): String {
    return when (provider) {
        AiProvider.LOCAL -> "Local AI"
        AiProvider.ANTHROPIC -> "Anthropic"
    }
}
