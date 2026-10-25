package dms.ui.document.tab

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.surface.LabelChip
import dms.ui.document.model.DocumentFactModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/** The details as a two-column grid of labels and values; a deadline carries its chip. */
@Composable
fun FactGrid(facts: List<DocumentFactModel>) {
    val colors = MaterialTheme.colorScheme
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        facts.forEach { fact ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
            ) {
                Text(
                    text = fact.label,
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(weight = 0.38f),
                )
                Row(
                    modifier = Modifier.weight(weight = 0.62f),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
                ) {
                    Text(
                        text = fact.value,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(weight = 1f, fill = false),
                    )
                    fact.chip?.let { chip ->
                        LabelChip(text = chip, color = if (fact.urgent) AppColors.statusWarning else colors.outline)
                    }
                }
            }
        }
    }
}
