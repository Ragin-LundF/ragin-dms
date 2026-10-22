package dms.application.port.plan

import dms.application.port.plan.model.RestructureRecordModel
import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.LibraryFolderModel

/** Applied restructures in `.dms-plans/<name>.yaml` of a library folder, replicated like the text store. */
interface PlanStore {

    suspend fun write(folder: LibraryFolderModel, record: RestructureRecordModel): StorageResult<Unit>

    /** The record called [name], or `null` if there is none or it cannot be read. */
    suspend fun read(folder: LibraryFolderModel, name: String): RestructureRecordModel?

    /** The names of the stored records, newest first. */
    suspend fun names(folder: LibraryFolderModel): List<String>
}
