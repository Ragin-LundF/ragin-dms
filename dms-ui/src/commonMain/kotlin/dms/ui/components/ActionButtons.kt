package dms.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import dms.ui.components.model.ActionButton
import dms.ui.theme.Spacing

/** A row of buttons; a disabled one explains in its tooltip why it cannot be used. */
@Composable
fun ActionButtons(buttons: List<ActionButton>, enabled: Boolean, busyReason: String?) {
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        buttons.forEach { button ->
            val reason = button.disabledReason ?: busyReason.takeUnless { enabled }
            ReasonTooltip(reason = reason) {
                OutlinedButton(onClick = button.onClick, enabled = reason == null) {
                    Text(text = button.label, maxLines = 1, softWrap = false)
                }
            }
        }
    }
}
