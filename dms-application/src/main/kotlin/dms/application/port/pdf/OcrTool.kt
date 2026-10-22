package dms.application.port.pdf

import dms.application.port.storage.model.StorageResult
import dms.domain.change.model.FileFactsModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/**
 * An OCR program installed on this device, such as `ocrmypdf`; nothing is bundled. It writes a
 * searchable copy of a scan to a target that must not exist yet.
 */
interface OcrTool {

    /** Whether the program is installed and can be run. */
    suspend fun available(): Boolean

    suspend fun ocr(
        folder: LibraryFolderModel,
        source: RelativePathModel,
        target: RelativePathModel,
    ): StorageResult<FileFactsModel>
}
