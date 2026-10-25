package dms.ui.library.folders.view.tree

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ViewSidebar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.theme.Spacing

/** The panel button that hides the folder tree, or shows it again while it is [hidden]. */
@Composable
fun TreeToggleButton(hidden: Boolean, onClick: () -> Unit) {
    IconButton(onClick = onClick) {
        Icon(
            imageVector = Icons.Rounded.ViewSidebar,
            contentDescription = if (hidden) "Show the folder tree" else "Hide the folder tree",
            modifier = Modifier.size(size = Spacing.icon),
        )
    }
}
