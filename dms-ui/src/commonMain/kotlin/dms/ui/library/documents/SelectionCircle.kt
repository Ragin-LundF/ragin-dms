package dms.ui.library.documents

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import dms.ui.theme.LibrarySizes

/** The circle that ticks a document for the bulk bar; it shows while [visible] or ticked. */
@Composable
fun SelectionCircle(
    title: String,
    selected: Boolean,
    visible: Boolean,
    onToggle: () -> Unit,
    modifier: Modifier = Modifier,
) {
    if (!visible && !selected) {
        Box(modifier = modifier.size(size = LibrarySizes.selectionCircle))
        return
    }
    val colors = MaterialTheme.colorScheme
    Surface(
        shape = CircleShape,
        color = if (selected) colors.primary else colors.surface,
        border = BorderStroke(
            width = LibrarySizes.circleBorder,
            color = if (selected) colors.primary else colors.outline,
        ),
        modifier = modifier
            .size(size = LibrarySizes.selectionCircle)
            .toggleable(value = selected, role = Role.Checkbox, onValueChange = { onToggle() })
            .semantics { contentDescription = "Select $title" },
    ) {
        if (selected) {
            Box(contentAlignment = Alignment.Center) {
                Icon(
                    imageVector = Icons.Rounded.Check,
                    contentDescription = null,
                    tint = colors.onPrimary,
                    modifier = Modifier.size(size = LibrarySizes.labelIcon),
                )
            }
        }
    }
}
