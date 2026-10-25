package dms.ui.shell.toolbar

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.components.ReasonTooltip
import dms.ui.shell.LocalCommandKey
import dms.ui.theme.Spacing

/** Looks like a search field and opens the command palette, which does the searching. */
@Composable
fun PaletteField(onOpen: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    val hint = "${LocalCommandKey.current}K"
    ReasonTooltip(reason = "Search and actions ($hint)") {
        Surface(
            onClick = onOpen,
            shape = MaterialTheme.shapes.small,
            color = colors.surfaceContainer,
            border = BorderStroke(width = Spacing.hairline, color = colors.outlineVariant),
            modifier = modifier.widthIn(max = Spacing.paletteFieldWidth),
        ) {
            Row(
                modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.small),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            ) {
                Icon(
                    imageVector = Icons.Rounded.Search,
                    contentDescription = null,
                    tint = colors.onSurfaceVariant,
                    modifier = Modifier.size(size = Spacing.icon),
                )
                Text(
                    text = "Search documents, tags, actions…",
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    modifier = Modifier.weight(weight = 1f),
                )
                Text(text = hint, style = MaterialTheme.typography.labelMedium, color = colors.onSurfaceVariant)
            }
        }
    }
}
