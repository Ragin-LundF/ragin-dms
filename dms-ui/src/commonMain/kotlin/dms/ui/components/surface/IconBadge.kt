package dms.ui.components.surface

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/** An icon on a soft tile of its own colour. */
@Composable
fun IconBadge(icon: ImageVector, tint: Color, modifier: Modifier = Modifier, size: Dp = 40.dp) {
    Box(
        modifier = modifier
            .size(size = size)
            .clip(shape = MaterialTheme.shapes.medium)
            .background(color = tint.copy(alpha = 0.14f)),
        contentAlignment = Alignment.Center,
    ) {
        Icon(imageVector = icon, contentDescription = null, tint = tint, modifier = Modifier.size(size = size * 0.55f))
    }
}
