package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel

/**
 * Moves or renames one document in every library folder. A rename that only changes the case of the
 * name is allowed; any other name clash is refused, never resolved by overwriting.
 */
class MoveDocument(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(documentId: DocumentIdModel, newPath: RelativePathModel): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        if (document.path == newPath) {
            return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
        }
        val violation = FileNamePolicy.check(name = newPath.fileName)
        if (violation != null) {
            return OperationOutcome.InvalidName(violation = violation)
        }
        // Names first: a clash with a catalogued document is a taken name, the same as one that differs by case.
        val free = names.isFree(library = library, target = newPath, moving = document.path)
        if (free is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = free)
        }
        if (free == StorageResult.Success(value = false)) {
            return OperationOutcome.NameTaken
        }
        val check = library.catalog.moveDocument(documentId = documentId, newPath = newPath)
        if (check is CatalogUpdate.Rejected) {
            return OperationOutcome.Rejected(reason = check.reason)
        }
        return move(library = library, documentId = documentId, from = document.path, to = newPath)
    }

    private suspend fun move(
        library: LibrarySnapshotModel,
        documentId: DocumentIdModel,
        from: RelativePathModel,
        to: RelativePathModel,
    ): OperationOutcome {
        val replication = replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> storage.move(folder = primary, from = from, to = to) },
            followUp = FolderOperation.Move(from = from, to = to),
        )
        if (replication is Replication.PrimaryFailed) {
            return OperationOutcome.StorageFailed(failure = replication.failure)
        }
        val pending = (replication as Replication.Done).pendingFolders
        val outcome = store.commit { it.moveDocument(documentId = documentId, newPath = to) }
            .withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.DOCUMENT_MOVED, subject = to.value)
        }
        return outcome
    }
}
