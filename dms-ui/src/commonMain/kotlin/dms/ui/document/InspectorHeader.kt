package dms.ui.document

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.components.surface.KindBadge
import dms.ui.preview.PageThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/** A large preview of the first page, the title and the file name of the document, and closing the inspector. */
@Composable
fun InspectorHeader(
    document: DocumentModel,
    folderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    onClose: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(height = Spacing.inspectorPreviewHeight)
                .clip(shape = MaterialTheme.shapes.medium)
                .background(color = MaterialTheme.colorScheme.surfaceContainer),
            contentAlignment = Alignment.Center,
        ) {
            val badge: @Composable () -> Unit = { KindBadge(kind = document.kind, size = 52.dp) }
            if (folderId == null) {
                badge()
            } else {
                PageThumbnail(
                    folderId = folderId,
                    path = document.path,
                    loader = thumbnails,
                    modifier = Modifier.fillMaxSize(),
                    fallback = badge,
                )
            }
        }
        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.Top) {
            Column(modifier = Modifier.weight(weight = 1f)) {
                Text(
                    text = document.metadata.title ?: document.path.fileName,
                    style = MaterialTheme.typography.titleMedium,
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
}
