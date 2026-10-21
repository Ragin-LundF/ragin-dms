package dms.application.resolution

import dms.application.common.ActivityRecorder
import dms.application.common.LibrarySnapshot
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.resolution.model.MoveDecision
import dms.application.sync.Replicator
import dms.domain.catalog.CatalogUpdate
import dms.domain.change.DetectedChange
import dms.domain.library.LibraryFolder

/** Applies a move made outside the tool to the other folders and the catalog, or undoes it. */
class ResolveMove(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(change: DetectedChange.MovedFile, decision: MoveDecision): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[change.documentId] ?: return OperationOutcome.NotFound
        if (document.path != change.previousPath) {
            return OperationOutcome.ScanOutdated
        }
        val folder = library.folders.find(id = change.folderId) ?: return OperationOutcome.NotFound
        return when (decision) {
            MoveDecision.APPLY -> apply(library = library, folder = folder, change = change)
            MoveDecision.UNDO -> undo(library = library, folder = folder, change = change)
        }
    }

    private suspend fun apply(
        library: LibrarySnapshot,
        folder: LibraryFolder,
        change: DetectedChange.MovedFile,
    ): OperationOutcome {
        val check = library.catalog.moveDocument(documentId = change.documentId, newPath = change.path)
        if (check is CatalogUpdate.Rejected) {
            return OperationOutcome.Rejected(reason = check.reason)
        }
        val primary = library.folders.primary
        if (folder != primary) {
            val moved = storage.move(folder = primary, from = change.previousPath, to = change.path)
            if (moved is StorageResult.Failure) {
                return OperationOutcome.StorageFailed(failure = moved)
            }
        }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders.filter { it != folder },
            operation = FolderOperation.Move(from = change.previousPath, to = change.path),
        )
        val outcome = store.commit { it.moveDocument(documentId = change.documentId, newPath = change.path) }
            .withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.DOCUMENT_MOVED, subject = change.path.value)
        }
        return outcome
    }

    private suspend fun undo(
        library: LibrarySnapshot,
        folder: LibraryFolder,
        change: DetectedChange.MovedFile,
    ): OperationOutcome {
        val moved = storage.move(folder = folder, from = change.path, to = change.previousPath)
        if (moved is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = moved)
        }
        activity.record(kind = ActivityKind.DOCUMENT_MOVED, subject = change.previousPath.value)
        return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
    }
}
