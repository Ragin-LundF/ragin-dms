package dms.ui.components

import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.theme.Spacing

/** One button in the style the view model chose, with its icon in front of the label. */
@Composable
fun StyledButton(button: ActionButton, enabled: Boolean, modifier: Modifier = Modifier) {
    val content: @Composable RowScope.() -> Unit = {
        button.icon?.let { icon ->
            Icon(imageVector = icon, contentDescription = null, modifier = Modifier.size(size = Spacing.icon))
            Spacer(modifier = Modifier.width(width = Spacing.small))
        }
        Text(text = button.label, maxLines = 1, softWrap = false)
    }
    when (button.style) {
        ButtonStyle.FILLED -> Button(
            onClick = button.onClick,
            enabled = enabled,
            modifier = modifier,
            content = content,
        )

        ButtonStyle.TONAL -> FilledTonalButton(
            onClick = button.onClick,
            enabled = enabled,
            modifier = modifier,
            content = content,
        )

        ButtonStyle.OUTLINED -> OutlinedButton(
            onClick = button.onClick,
            enabled = enabled,
            modifier = modifier,
            content = content,
        )

        ButtonStyle.TEXT -> TextButton(
            onClick = button.onClick,
            enabled = enabled,
            modifier = modifier,
            content = content,
        )
    }
}
