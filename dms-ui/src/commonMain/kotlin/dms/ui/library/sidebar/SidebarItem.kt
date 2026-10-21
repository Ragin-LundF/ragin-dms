package dms.ui.library.sidebar

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.theme.Spacing

/** One selectable line of the sidebar, indented by [depth], with an optional count. */
@Composable
fun SidebarItem(label: String, count: Int?, depth: Int, selected: Boolean, onClick: () -> Unit) {
    val background = if (selected) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(color = background, shape = MaterialTheme.shapes.small)
            .clickable(onClick = onClick)
            .padding(
                start = Spacing.small + Spacing.medium * depth,
                end = Spacing.small,
                top = Spacing.xsmall,
                bottom = Spacing.xsmall,
            ),
    ) {
        Text(
            text = label,
            maxLines = 1,
            softWrap = false,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(weight = 1f),
        )
        Spacer(modifier = Modifier.padding(start = Spacing.small))
        count?.let { Text(text = it.toString(), style = MaterialTheme.typography.bodySmall) }
    }
}
