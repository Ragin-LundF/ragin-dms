package dms.application.preview.model

import dms.application.port.ai.model.PageImageModel

sealed interface ThumbnailOutcome {

    data class Rendered(val image: PageImageModel) : ThumbnailOutcome

    /** The file has no picture, or it cannot be read. */
    data object NoPicture : ThumbnailOutcome

    /** No library is open, or the folder is not one of its folders. */
    data object FolderNotFound : ThumbnailOutcome
}
