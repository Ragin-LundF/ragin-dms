package dms.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.model.ActionButton
import dms.ui.theme.Spacing

/** A row of buttons that wraps when narrow; a disabled one explains in its tooltip why it cannot be used. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ActionButtons(
    buttons: List<ActionButton>,
    enabled: Boolean,
    busyReason: String?,
    modifier: Modifier = Modifier,
) {
    FlowRow(
        modifier = modifier,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        buttons.forEach { button ->
            val reason = button.disabledReason ?: busyReason.takeUnless { enabled }
            ReasonTooltip(reason = reason) {
                StyledButton(button = button, enabled = reason == null)
            }
        }
    }
}
