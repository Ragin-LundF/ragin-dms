package dms.ui.library.documents.list

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.components.surface.KindStyles
import dms.ui.library.DocumentColumns
import dms.ui.preview.PageThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/** A small picture of the document's first page; the kind's icon while it loads or if there is none. */
@Composable
fun RowThumbnail(document: DocumentModel, folderId: LibraryFolderIdModel?, loader: ThumbnailLoader) {
    val modifier = Modifier
        .size(width = DocumentColumns.thumbnailWidth, height = DocumentColumns.thumbnailHeight)
        .clip(shape = MaterialTheme.shapes.extraSmall)
    if (folderId == null) {
        KindTile(document = document, modifier = modifier)
        return
    }
    PageThumbnail(folderId = folderId, path = document.path, loader = loader, modifier = modifier) {
        KindTile(document = document, modifier = modifier)
    }
}

@Composable
private fun KindTile(document: DocumentModel, modifier: Modifier) {
    val color = KindStyles.color(kind = document.kind)
    Box(modifier = modifier.background(color = color.copy(alpha = 0.14f)), contentAlignment = Alignment.Center) {
        Icon(
            imageVector = KindStyles.icon(kind = document.kind),
            contentDescription = KindStyles.label(kind = document.kind),
            tint = color,
            modifier = Modifier.size(size = Spacing.medium),
        )
    }
}
