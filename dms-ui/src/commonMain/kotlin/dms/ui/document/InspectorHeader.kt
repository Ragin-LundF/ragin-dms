package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.domain.document.model.DocumentModel
import dms.ui.components.surface.KindBadge
import dms.ui.theme.Spacing

/** The kind badge, the title and the file name of the inspected document, and closing the inspector. */
@Composable
fun InspectorHeader(document: DocumentModel, onClose: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.Top,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        KindBadge(kind = document.kind, size = 52.dp)
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(
                text = document.metadata.title ?: document.path.fileName,
                style = MaterialTheme.typography.titleLarge,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = document.path.fileName,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        IconButton(onClick = onClose) {
            Icon(imageVector = Icons.Rounded.Close, contentDescription = "Close")
        }
    }
}
