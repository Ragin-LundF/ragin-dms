package dms.ui.preview

import androidx.compose.foundation.Image
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/**
 * A small picture of the file at [path] in [folderId], loaded in the background; clicking it calls
 * [onClick]. Until the picture is there, and for files without one, [fallback] is shown instead.
 */
@Composable
fun FileThumbnail(
    folderId: LibraryFolderId,
    path: RelativePath,
    loader: ThumbnailLoader,
    onClick: () -> Unit,
    fallback: @Composable () -> Unit,
) {
    val thumbnail by produceState<ImageBitmap?>(initialValue = null, key1 = folderId, key2 = path) {
        value = loader.load(folderId = folderId, path = path)
    }
    val image = thumbnail
    if (image == null) {
        fallback()
        return
    }
    val shape = MaterialTheme.shapes.small
    Image(
        bitmap = image,
        contentDescription = "Preview of ${path.fileName}",
        contentScale = ContentScale.Fit,
        modifier = Modifier
            .size(width = 64.dp, height = 88.dp)
            .clip(shape = shape)
            .border(width = 1.dp, color = MaterialTheme.colorScheme.outlineVariant, shape = shape)
            .clickable(onClick = onClick),
    )
}
