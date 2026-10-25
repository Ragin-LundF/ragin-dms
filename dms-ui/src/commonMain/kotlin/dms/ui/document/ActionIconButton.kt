package dms.ui.document

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedIconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.ReasonTooltip
import dms.ui.components.model.ActionButtonModel
import dms.ui.theme.Spacing

/** An action as an outlined icon; its tooltip names it, or says why it cannot be used now. */
@Composable
fun ActionIconButton(button: ActionButtonModel) {
    val icon = checkNotNull(value = button.icon) { "An icon button needs an icon: ${button.label}" }
    ReasonTooltip(reason = button.disabledReason ?: button.label) {
        OutlinedIconButton(
            onClick = button.onClick,
            enabled = button.disabledReason == null,
            shape = MaterialTheme.shapes.medium,
            border = BorderStroke(width = Spacing.hairline, color = MaterialTheme.colorScheme.outlineVariant),
        ) {
            Icon(imageVector = icon, contentDescription = button.label, modifier = Modifier.size(size = Spacing.icon))
        }
    }
}
