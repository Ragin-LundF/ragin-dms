package dms.ui.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.OpenInFull
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable

/** Opens the full view of a dashboard card. */
@Composable
fun FullViewButton(label: String, onClick: () -> Unit) {
    ReasonTooltip(reason = label) {
        IconButton(onClick = onClick) {
            Icon(
                imageVector = Icons.Rounded.OpenInFull,
                contentDescription = label,
            )
        }
    }
}
