package dms.ui.document.quicklook

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
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
import dms.ui.components.ReasonTooltip
import dms.ui.components.surface.KindBadge
import dms.ui.document.edit.InlineTextEditor
import dms.ui.preview.PageThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing


/**
 * A large preview of the first page, the title, which a click edits in place ([editingTitle]), and
 * the file's path in the library.
 */
@Composable
fun QuickLookHeader(
    document: DocumentModel,
    folderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    editingTitle: Boolean,
    busyReason: String?,
    onEditTitle: () -> Unit,
    onSaveTitle: (String) -> Unit,
    onCancel: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        PagePreview(document = document, folderId = folderId, thumbnails = thumbnails)
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline * 2)) {
            if (editingTitle) {
                InlineTextEditor(
                    initial = document.metadata.title.orEmpty(),
                    description = "Title",
                    onSave = onSaveTitle,
                    onCancel = onCancel,
                )
            } else {
                ReasonTooltip(reason = busyReason) {
                    Text(
                        text = document.metadata.title ?: document.path.fileName,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.clickable(enabled = busyReason == null, onClick = onEditTitle),
                    )
                }
            }
            Text(
                text = document.path.value,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.outline,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun PagePreview(document: DocumentModel, folderId: LibraryFolderIdModel?, thumbnails: ThumbnailLoader) {
    Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
        Box(
            modifier = Modifier
                .size(width = LibrarySizes.quickLookPreviewWidth, height = LibrarySizes.quickLookPreviewHeight)
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
    }
}
