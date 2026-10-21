package dms.ui.library

import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.theme.Spacing

/** One cell of a list row: a single line, cut off with an ellipsis. */
@Composable
fun RowScope.ListCell(text: String, weight: Float, style: TextStyle) {
    Text(
        text = text,
        style = style,
        maxLines = 1,
        softWrap = false,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.weight(weight = weight).padding(end = Spacing.small),
    )
}
