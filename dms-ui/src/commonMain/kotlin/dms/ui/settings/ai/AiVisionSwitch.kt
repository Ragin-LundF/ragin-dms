package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.theme.Spacing

/** The switch that makes the AI read PDFs as rendered pages even when they carry text. */
@Composable
fun AiVisionSwitch(checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(text = "Read PDFs as images", style = MaterialTheme.typography.titleSmall)
            Text(
                text = "Sends rendered pages even when a PDF has text. Helps with poor scanner text; " +
                    "needs a vision model and is slower.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Switch(checked = checked, onCheckedChange = onCheckedChange)
    }
}
