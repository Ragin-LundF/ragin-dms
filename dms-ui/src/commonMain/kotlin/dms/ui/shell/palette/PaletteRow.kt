package dms.ui.shell.palette

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.shell.LocalCommandKey
import dms.ui.shell.palette.model.PaletteItemModel
import dms.ui.theme.Spacing

/**
 * One line of the palette: its label, what it is or where it was found, and a count or a shortcut. An
 * action that waits for its lane is dimmed and names the reason instead.
 */
@Composable
fun PaletteRow(item: PaletteItemModel, selected: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val waiting = (item as? PaletteItemModel.Run)?.disabledReason != null
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = MaterialTheme.shapes.small)
            .background(color = if (selected) colors.primaryContainer else Color.Transparent)
            .alpha(alpha = if (waiting) WAITING_ALPHA else 1f)
            .clickable(enabled = !waiting, onClick = onClick)
            .padding(horizontal = Spacing.medium, vertical = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(
                text = item.label,
                style = MaterialTheme.typography.bodyMedium,
                color = if (selected) colors.onPrimaryContainer else colors.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            detailOf(item = item)?.let { detail ->
                Text(
                    text = detail,
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        hintOf(item = item, commandKey = LocalCommandKey.current)?.let { hint ->
            Text(text = hint, style = MaterialTheme.typography.labelMedium, color = colors.onSurfaceVariant)
        }
    }
}

private fun detailOf(item: PaletteItemModel): String? {
    return when (item) {
        is PaletteItemModel.OpenDocument -> item.detail
        is PaletteItemModel.GoTo -> item.kind
        is PaletteItemModel.Run -> item.disabledReason
        is PaletteItemModel.AskLibrary -> "Answers come from your documents, with sources"
    }
}

private fun hintOf(item: PaletteItemModel, commandKey: String): String? {
    return when (item) {
        is PaletteItemModel.OpenDocument -> null
        is PaletteItemModel.GoTo -> item.count?.toString()
        is PaletteItemModel.Run -> item.action.shortcutKey?.let { key ->
            commandKey + (if (item.action.shift) "⇧" else "") + key
        }
        is PaletteItemModel.AskLibrary -> "$commandKey↵"
    }
}

private const val WAITING_ALPHA = 0.5f
