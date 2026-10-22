package dms.ui.library.documents

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.document.model.DocumentModel

/** The title of a document, or its file name, above its path. */
@Composable
fun NameCell(document: DocumentModel, modifier: Modifier = Modifier) {
    Column(modifier = modifier) {
        Text(
            text = document.metadata.title ?: document.path.fileName,
            style = MaterialTheme.typography.bodyLarge,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = document.path.value,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
