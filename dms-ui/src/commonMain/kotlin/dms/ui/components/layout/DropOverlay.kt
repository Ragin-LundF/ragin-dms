package dms.ui.components.layout

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudUpload
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.surface.IconBadge
import dms.ui.theme.Spacing

/** The highlighted area that tells where dropped files will go. */
@Composable
fun DropOverlay(label: String) {
    val colors = MaterialTheme.colorScheme
    Surface(
        modifier = Modifier.fillMaxSize().padding(all = Spacing.large),
        shape = MaterialTheme.shapes.extraLarge,
        color = colors.primaryContainer.copy(alpha = 0.92f),
        border = BorderStroke(width = 2.dp, color = colors.primary),
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(space = Spacing.medium, alignment = Alignment.CenterVertically),
        ) {
            IconBadge(icon = Icons.Rounded.CloudUpload, tint = colors.primary, size = 72.dp)
            Text(
                text = "Drop to add",
                style = MaterialTheme.typography.headlineSmall,
                color = colors.onPrimaryContainer,
            )
            Text(text = label, style = MaterialTheme.typography.bodyLarge, color = colors.onPrimaryContainer)
        }
    }
}
