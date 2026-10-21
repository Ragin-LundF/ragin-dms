package dms.ui.components.surface

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.theme.Spacing

/** A small read-only chip for a category or a tag, tinted in [color]. */
@Composable
fun LabelChip(text: String, color: Color, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurface,
        maxLines = 1,
        softWrap = false,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
            .background(color = color.copy(alpha = 0.14f), shape = CircleShape)
            .padding(horizontal = Spacing.small, vertical = Spacing.hairline * 2),
    )
}
