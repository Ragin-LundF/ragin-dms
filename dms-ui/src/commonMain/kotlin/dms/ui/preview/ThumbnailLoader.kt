package dms.ui.preview

import androidx.compose.ui.graphics.ImageBitmap
import dms.application.preview.model.ThumbnailOutcome
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.ui.PlatformActions
import dms.ui.service.AppServices

/** Loads the thumbnails of library files for the screen. */
class ThumbnailLoader(
    private val services: AppServices,
    private val platform: PlatformActions,
) {

    /** The thumbnail of [path] in [folderId], or `null` if the file has none. */
    suspend fun load(folderId: LibraryFolderIdModel, path: RelativePathModel): ImageBitmap? {
        return when (val outcome = services.library.thumbnails(folderId = folderId, path = path)) {
            is ThumbnailOutcome.Rendered -> platform.decodeImage(bytes = outcome.image.bytes)
            ThumbnailOutcome.NoPicture,
            ThumbnailOutcome.FolderNotFound,
            -> null
        }
    }
}
