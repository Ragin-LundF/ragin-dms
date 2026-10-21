package dms.ui.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloseFullscreen
import androidx.compose.material.icons.rounded.OpenInFull
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable

/** Opens a dashboard card in a full view, or with [full] goes back to the card. */
@Composable
fun FullViewButton(label: String, full: Boolean, onClick: () -> Unit) {
    ReasonTooltip(reason = label) {
        IconButton(onClick = onClick) {
            Icon(
                imageVector = if (full) Icons.Rounded.CloseFullscreen else Icons.Rounded.OpenInFull,
                contentDescription = label,
            )
        }
    }
}
