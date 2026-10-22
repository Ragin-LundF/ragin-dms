package dms.application.port.ai

import dms.application.port.ai.model.PageImageModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/** Small pictures of files for the screen: the first page of a PDF, or a photo. Never modifies the file. */
interface DocumentPreviewer {

    /** A picture at most [maxSide] pixels on its long side, or `null` for a file without one. */
    suspend fun thumbnail(folder: LibraryFolderModel, path: RelativePathModel, maxSide: Int): PageImageModel?
}
