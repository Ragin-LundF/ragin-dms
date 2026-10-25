package dms.ui.shell.toolbar

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.theme.Spacing

/**
 * The bar along the top of every screen: the screen's [title] on the left, the field that opens the
 * command palette in the middle, and the screen's [actions] on the right.
 */
@Composable
fun WindowToolbar(title: String, onOpenPalette: () -> Unit, actions: @Composable RowScope.() -> Unit) {
    val colors = MaterialTheme.colorScheme
    Surface(color = colors.surfaceContainerLow, modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.height(height = Spacing.toolbarHeight).padding(horizontal = Spacing.screen),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(weight = 1f),
            )
            Box(modifier = Modifier.weight(weight = 1.4f), contentAlignment = Alignment.Center) {
                PaletteField(onOpen = onOpenPalette, modifier = Modifier.fillMaxWidth())
            }
            Row(
                modifier = Modifier.weight(weight = 1f),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.small, alignment = Alignment.End),
                content = actions,
            )
        }
    }
    HorizontalDivider(color = colors.outlineVariant)
}
