package dms.ui.library.documents

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.components.surface.LabelChip
import dms.ui.library.DocumentColumns

/** The first two tags as chips and how many more there are. */
@Composable
fun TagChips(tags: List<TagNameModel>, modifier: Modifier = Modifier) {
    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(space = DocumentColumns.chipGap)) {
        tags.take(n = 2).forEach { LabelChip(text = it.value, color = MaterialTheme.colorScheme.secondary) }
        if (tags.size > 2) {
            Text(text = "+${tags.size - 2}", style = MaterialTheme.typography.labelMedium)
        }
    }
}
