package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import dms.ui.theme.Spacing

/**
 * The models the server offered in the last test; picking one fills the model field. The first chip,
 * [firstLabel], leaves the field blank: the server's first loaded model, or Anthropic's default one.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun AiModelChips(models: List<String>, selected: String, firstLabel: String, onSelect: (String) -> Unit) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        FilterChip(
            selected = selected.isBlank(),
            onClick = { onSelect("") },
            label = { Text(text = firstLabel) },
        )
        models.forEach { model ->
            FilterChip(
                selected = selected == model,
                onClick = { onSelect(model) },
                label = { Text(text = model, softWrap = false) },
            )
        }
    }
}
