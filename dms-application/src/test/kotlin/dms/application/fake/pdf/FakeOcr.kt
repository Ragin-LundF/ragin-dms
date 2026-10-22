package dms.application.fake.pdf

import dms.application.fake.storage.FakeFolders
import dms.application.port.pdf.OcrTool
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/** An OCR program that is installed when [installed] says so, and marks every page as having text. */
class FakeOcr(private val folders: FakeFolders) : OcrTool {

    var installed: Boolean = false

    override suspend fun available(): Boolean {
        return installed
    }

    override suspend fun ocr(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel> {
        val content = folders.content(folder = folder, path = source.value)
            ?: return StorageResult.Failure(reason = StorageFailure.NOT_FOUND)
        val recognized = content.split('|').joinToString(separator = "|") { "$it#ocr" }
        folders.put(folder = folder, path = target.value, content = recognized)
        return StorageResult.Success(value = FakeFolders.factsOf(content = recognized))
    }
}
