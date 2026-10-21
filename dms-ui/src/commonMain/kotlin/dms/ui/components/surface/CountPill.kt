package dms.ui.components.surface

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.theme.Spacing

/** A small rounded count, as next to sidebar entries and filters. */
@Composable
fun CountPill(count: Int, selected: Boolean = false) {
    val colors = MaterialTheme.colorScheme
    Text(
        text = count.toString(),
        style = MaterialTheme.typography.labelSmall,
        color = if (selected) colors.onPrimary else colors.onSurfaceVariant,
        maxLines = 1,
        softWrap = false,
        modifier = Modifier
            .background(color = if (selected) colors.primary else colors.surfaceContainerHigh, shape = CircleShape)
            .padding(horizontal = Spacing.small, vertical = Spacing.hairline),
    )
}
