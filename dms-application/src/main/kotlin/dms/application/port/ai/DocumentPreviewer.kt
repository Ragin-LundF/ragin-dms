package dms.application.port.ai

import dms.application.port.ai.model.PageImage
import dms.domain.library.LibraryFolder
import dms.domain.library.RelativePath

/** Small pictures of files for the screen: the first page of a PDF, or a photo. Never modifies the file. */
interface DocumentPreviewer {

    /** A picture at most [maxSide] pixels on its long side, or `null` for a file without one. */
    suspend fun thumbnail(folder: LibraryFolder, path: RelativePath, maxSide: Int): PageImage?
}
