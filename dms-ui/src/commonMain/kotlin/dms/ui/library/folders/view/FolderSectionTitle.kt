package dms.ui.library.folders.view

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.theme.Spacing

/** The heading of a section of the folder view with its [count], and a [hint] at the end of the line. */
@Composable
fun FolderSectionTitle(title: String, count: Int, hint: String? = null) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.SemiBold,
            color = colors.onSurfaceVariant,
        )
        Text(text = "$count", style = MaterialTheme.typography.labelMedium, color = colors.outline)
        hint?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.labelMedium,
                color = colors.outline,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
                textAlign = TextAlign.End,
                modifier = Modifier.weight(weight = 1f),
            )
        }
    }
}
