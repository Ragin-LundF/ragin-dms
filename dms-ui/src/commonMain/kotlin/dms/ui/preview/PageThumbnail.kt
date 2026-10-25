package dms.ui.preview

import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * The picture of the file at [path] in [folderId], loaded in the background and fitted into
 * [modifier]'s size. Until the picture is there, and for files without one, [fallback] is shown instead.
 */
@Composable
fun PageThumbnail(
    folderId: LibraryFolderIdModel,
    path: RelativePathModel,
    loader: ThumbnailLoader,
    modifier: Modifier = Modifier,
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
    Image(
        bitmap = image,
        contentDescription = "Preview of ${path.fileName}",
        contentScale = ContentScale.Fit,
        modifier = modifier,
    )
}
