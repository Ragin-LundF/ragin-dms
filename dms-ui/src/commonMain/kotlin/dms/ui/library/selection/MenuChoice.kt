package dms.ui.library.selection

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.theme.Spacing

/** One entry of a bulk bar menu, indented by [depth] levels. */
@Composable
internal fun MenuChoice(text: String, depth: Int, italic: Boolean = false, onClick: () -> Unit) {
    DropdownMenuItem(
        text = {
            Text(
                text = text,
                fontStyle = if (italic) FontStyle.Italic else FontStyle.Normal,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
        },
        contentPadding = PaddingValues(start = Spacing.medium + Spacing.large * depth, end = Spacing.medium),
        onClick = onClick,
    )
}
