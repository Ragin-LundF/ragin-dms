package dms.application.fake.storage

import dms.application.port.plan.PlanStore
import dms.application.port.plan.model.RestructureRecordModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel

/** `.dms-plans/` of every library folder, in memory; folders in [unreachable] fail like in [FakeFolders]. */
class InMemoryPlanStore(private val unreachable: Set<LibraryFolderIdModel>) : PlanStore {

    private val records = mutableMapOf<LibraryFolderIdModel, MutableMap<String, RestructureRecordModel>>()

    fun recordsIn(folder: LibraryFolderModel): Map<String, RestructureRecordModel> {
        return records[folder.id].orEmpty()
    }

    override suspend fun write(folder: LibraryFolderModel, record: RestructureRecordModel): StorageResult<Unit> {
        if (folder.id in unreachable) {
            return StorageResult.Failure(reason = StorageFailure.FOLDER_UNREACHABLE)
        }
        records.getOrPut(key = folder.id) { mutableMapOf() }[record.name] = record
        return StorageResult.Success(value = Unit)
    }

    override suspend fun read(folder: LibraryFolderModel, name: String): RestructureRecordModel? {
        if (folder.id in unreachable) {
            return null
        }
        return records[folder.id]?.get(key = name)
    }

    override suspend fun names(folder: LibraryFolderModel): List<String> {
        return records[folder.id]?.keys.orEmpty().sortedDescending()
    }
}
