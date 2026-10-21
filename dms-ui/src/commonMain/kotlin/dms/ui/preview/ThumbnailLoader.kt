package dms.ui.preview

import androidx.compose.ui.graphics.ImageBitmap
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath
import dms.ui.PlatformActions
import dms.ui.service.AppServices

/** Loads the thumbnails of library files for the screen. */
class ThumbnailLoader(
    private val services: AppServices,
    private val platform: PlatformActions,
) {

    /** The thumbnail of [path] in [folderId], or `null` if the file has none. */
    suspend fun load(folderId: LibraryFolderId, path: RelativePath): ImageBitmap? {
        val image = services.library.thumbnails(folderId = folderId, path = path) ?: return null
        return platform.decodeImage(bytes = image.bytes)
    }
}
