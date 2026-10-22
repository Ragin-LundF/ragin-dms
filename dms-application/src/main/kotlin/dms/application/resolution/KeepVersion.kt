package dms.application.resolution

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.application.sync.Replicator
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Makes the version of a path in one folder the version of every folder: for a content
 * conflict, or to accept or revert a file modified outside the tool. The replaced versions go to the
 * system trash, and the catalog records the kept content. Requires the user's confirmation.
 */
class KeepVersion(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(
        path: RelativePathModel,
        sourceFolderId: LibraryFolderIdModel,
        confirmed: Boolean,
    ): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val source = library.folders.find(id = sourceFolderId) ?: return OperationOutcome.NotFound
        val primary = library.folders.primary
        val kept = if (source == primary) {
            storage.readFacts(folder = primary, path = path)
        } else {
            storage.copyFile(from = source, to = primary, path = path, replaceExisting = true)
        }
        if (kept is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = kept)
        }
        val facts = (kept as StorageResult.Success).value
        val contentHash = facts.contentHash
            ?: return OperationOutcome.StorageFailed(failure = StorageResult.Failure(reason = StorageFailure.IO_ERROR))
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders.filter { it != source },
            operation = FolderOperation.CopyFromPrimary(path = path, replaceExisting = true),
        )
        activity.record(kind = ActivityKind.CONFLICT_RESOLVED, subject = path.value)
        val document = library.catalog.findByPath(path = path)
        if (document == null || document.contentHash == contentHash) {
            return OperationOutcome.Done(catalog = library.catalog, pendingFolders = pending)
        }
        return store.commit {
            it.updateContent(documentId = document.id, contentHash = contentHash, size = facts.size)
        }.withPendingFolders(folders = pending)
    }
}
