package dms.application.fake.ai

import dms.application.port.ai.DocumentPreviewer
import dms.application.port.ai.model.PageImageModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/** Draws the [pictures] by file path; every other file has none. Records each request in [requests]. */
class FakePreviewer : DocumentPreviewer {

    val pictures: MutableMap<RelativePathModel, PageImageModel> = mutableMapOf()
    val requests: MutableList<Request> = mutableListOf()

    override suspend fun thumbnail(folder: LibraryFolderModel, path: RelativePathModel, maxSide: Int): PageImageModel? {
        requests += Request(folder = folder, path = path, maxSide = maxSide)
        return pictures[path]
    }

    data class Request(val folder: LibraryFolderModel, val path: RelativePathModel, val maxSide: Int)
}
