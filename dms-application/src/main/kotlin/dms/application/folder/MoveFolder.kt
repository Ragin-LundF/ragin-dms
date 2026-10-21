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
import dms.domain.catalog.CatalogUpdate
import dms.domain.library.FileNamePolicy
import dms.domain.library.RelativePath

/**
 * Moves or renames a folder with everything in it, in every library folder, and moves the paths of
 * the documents inside it in the catalog.
 */
class MoveFolder(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(from: RelativePath, to: RelativePath): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val violation = FileNamePolicy.check(name = to.fileName)
        if (violation != null) {
            return OperationOutcome.InvalidName(violation = violation)
        }
        val check = library.catalog.moveFolder(from = from, to = to)
        if (check is CatalogUpdate.Rejected) {
            return OperationOutcome.Rejected(reason = check.reason)
        }
        val free = names.isFree(library = library, target = to, moving = from)
        if (free is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = free)
        }
        if (free == StorageResult.Success(value = false)) {
            return OperationOutcome.NameTaken
        }
        val replication = replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> storage.move(folder = primary, from = from, to = to) },
            followUp = FolderOperation.Move(from = from, to = to),
        )
        if (replication is Replication.PrimaryFailed) {
            return OperationOutcome.StorageFailed(failure = replication.failure)
        }
        val pending = (replication as Replication.Done).pendingFolders
        val outcome = store.commit { it.moveFolder(from = from, to = to) }.withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.FOLDER_MOVED, subject = to.value)
        }
        return outcome
    }
}
