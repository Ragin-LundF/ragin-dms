package dms.application.port.export

import dms.application.port.export.model.ExportBundleModel
import dms.application.port.export.model.ExportTargetModel
import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.LibraryFolderModel

/**
 * Writes copies of library files outside the library, for example for the tax adviser. It only reads
 * the library; every copy is verified by hash.
 */
interface ExportWriter {

    /** `true` if [directory], an absolute path of this device, is one of [folders] or lies inside one. */
    fun isInside(directory: String, folders: List<LibraryFolderModel>): Boolean

    /** Writes [bundle], copying from [source], and returns where it was written. */
    suspend fun write(
        source: LibraryFolderModel,
        bundle: ExportBundleModel,
        target: ExportTargetModel,
    ): StorageResult<String>

    /** Writes one text file named [name] into [directory] (a free name is chosen) and returns its location. */
    suspend fun writeText(directory: String, name: String, text: String): StorageResult<String>
}
