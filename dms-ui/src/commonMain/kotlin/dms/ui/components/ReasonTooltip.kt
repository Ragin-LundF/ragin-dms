package dms.ui.components

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.PlainTooltip
import androidx.compose.material3.Text
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.material3.TooltipBox
import androidx.compose.material3.TooltipDefaults
import androidx.compose.material3.rememberTooltipState
import androidx.compose.runtime.Composable

/** Shows [reason] as a tooltip over [content]; without a reason, [content] alone. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReasonTooltip(reason: String?, content: @Composable () -> Unit) {
    if (reason == null) {
        content()
        return
    }
    TooltipBox(
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(
            positioning = TooltipAnchorPosition.Above,
        ),
        tooltip = { PlainTooltip { Text(text = reason) } },
        state = rememberTooltipState(),
        content = content,
    )
}
