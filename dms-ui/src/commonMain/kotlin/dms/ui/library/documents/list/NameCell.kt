package dms.ui.library.documents.list

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.document.model.DocumentModel

/**
 * The title of a document above its file name; a document without a title shows its file name above
 * its folder instead, so no name shows twice.
 */
@Composable
fun NameCell(document: DocumentModel, modifier: Modifier = Modifier) {
    val title = document.metadata.title
    val below = if (title == null) {
        document.path.parent?.value?.ifEmpty { null } ?: "Top level"
    } else {
        document.path.fileName
    }
    Column(modifier = modifier) {
        Text(
            text = title ?: document.path.fileName,
            style = MaterialTheme.typography.bodyMedium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = below,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
