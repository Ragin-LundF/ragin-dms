package dms.ui.library.filter

import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.InputChip
import androidx.compose.material3.InputChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.library.filter.model.ActiveFilter
import dms.ui.theme.Spacing

/** One active filter as a chip; a click removes it. A text filter spins while the content search runs. */
@Composable
fun RemovableFilterChip(filter: ActiveFilter, searching: Boolean, onRemove: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val label = ActiveFilters.label(filter = filter)
    val spinning = searching && filter is ActiveFilter.Text && filter.inContent
    InputChip(
        selected = true,
        onClick = onRemove,
        label = { Text(text = label, maxLines = 1, softWrap = false, overflow = TextOverflow.Ellipsis) },
        leadingIcon = if (spinning) {
            {
                CircularProgressIndicator(
                    modifier = Modifier.size(size = Spacing.icon),
                    strokeWidth = Spacing.hairline * 2,
                )
            }
        } else {
            null
        },
        trailingIcon = {
            Icon(
                imageVector = Icons.Rounded.Close,
                contentDescription = "Remove $label",
                modifier = Modifier.size(size = Spacing.icon),
            )
        },
        shape = CircleShape,
        colors = InputChipDefaults.inputChipColors(
            selectedContainerColor = colors.primaryContainer,
            selectedLabelColor = colors.onPrimaryContainer,
            selectedLeadingIconColor = colors.onPrimaryContainer,
            selectedTrailingIconColor = colors.onPrimaryContainer,
        ),
        border = null,
    )
}
