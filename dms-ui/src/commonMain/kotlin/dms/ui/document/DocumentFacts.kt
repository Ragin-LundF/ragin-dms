package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.domain.document.Document
import dms.ui.components.FileSizes
import dms.ui.components.surface.KindStyles
import dms.ui.dashboard.ActivityTexts
import dms.ui.theme.Spacing

/** Where the document lives, its type, its size and when it was added. */
@Composable
fun DocumentFacts(document: Document) {
    val facts = listOf(
        "Folder" to (document.path.parent?.value?.ifEmpty { "Top level" } ?: "Top level"),
        "Type" to KindStyles.label(kind = document.kind),
        "Size" to FileSizes.format(bytes = document.size),
        "Added" to ActivityTexts.time(instant = document.addedAt),
    )
    Surface(
        modifier = Modifier.fillMaxWidth().padding(top = Spacing.small),
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surfaceContainerLow,
    ) {
        Column(
            modifier = Modifier.padding(all = Spacing.medium),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            facts.forEach { (label, value) ->
                Row {
                    Text(
                        text = label,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.width(width = 72.dp),
                    )
                    Text(
                        text = value,
                        style = MaterialTheme.typography.bodySmall,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}
