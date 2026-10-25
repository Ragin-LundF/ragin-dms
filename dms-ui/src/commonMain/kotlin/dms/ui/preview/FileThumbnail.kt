package dms.ui.preview

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * A small picture of the file at [path] in [folderId], loaded in the background; clicking it calls
 * [onClick]. Until the picture is there, and for files without one, [fallback] is shown instead.
 */
@Composable
fun FileThumbnail(
    folderId: LibraryFolderIdModel,
    path: RelativePathModel,
    loader: ThumbnailLoader,
    onClick: () -> Unit,
    fallback: @Composable () -> Unit,
) {
    val shape = MaterialTheme.shapes.small
    PageThumbnail(
        folderId = folderId,
        path = path,
        loader = loader,
        modifier = Modifier
            .size(width = 64.dp, height = 88.dp)
            .clip(shape = shape)
            .border(width = 1.dp, color = MaterialTheme.colorScheme.outlineVariant, shape = shape)
            .clickable(onClick = onClick),
        fallback = fallback,
    )
}
