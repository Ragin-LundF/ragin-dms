package dms.application.folder

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel

/** Creates an empty folder in every library folder. The catalog does not change. */
class CreateFolder(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(path: RelativePathModel): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val violation = FileNamePolicy.check(name = path.fileName)
        if (violation != null) {
            return OperationOutcome.InvalidName(violation = violation)
        }
        val free = names.isFree(library = library, target = path, moving = null)
        if (free is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = free)
        }
        if (free == StorageResult.Success(value = false)) {
            return OperationOutcome.NameTaken
        }
        val replication = replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> storage.createFolder(folder = primary, path = path) },
            followUp = FolderOperation.CreateFolder(path = path),
        )
        return when (replication) {
            is Replication.PrimaryFailed -> OperationOutcome.StorageFailed(failure = replication.failure)
            is Replication.Done -> {
                activity.record(kind = ActivityKind.FOLDER_CREATED, subject = path.value)
                OperationOutcome.Done(catalog = library.catalog, pendingFolders = replication.pendingFolders)
            }
        }
    }
}
