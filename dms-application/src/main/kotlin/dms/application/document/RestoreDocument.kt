package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.sync.Replicator
import dms.domain.change.DetectedChange

/**
 * Copies a catalogued file back into the folders it is missing from, taking it from a folder that
 * still has it. The default answer to a file deleted outside the tool.
 */
class RestoreDocument(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(change: DetectedChange.MissingFile): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[change.documentId] ?: return OperationOutcome.NotFound
        if (document.path != change.path) {
            return OperationOutcome.ScanOutdated
        }
        val primary = library.folders.primary
        if (primary.id in change.missingIn) {
            val source = library.folders.folders.firstOrNull { it.id in change.presentIn }
                ?: return OperationOutcome.NotFound
            val copied = storage.copyFile(from = source, to = primary, path = change.path, replaceExisting = false)
            if (copied is StorageResult.Failure) {
                return OperationOutcome.StorageFailed(failure = copied)
            }
        }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders.filter { it.id in change.missingIn },
            operation = FolderOperation.CopyFromPrimary(path = change.path),
        )
        activity.record(kind = ActivityKind.DOCUMENT_RESTORED, subject = change.path.value)
        return OperationOutcome.Done(catalog = library.catalog, pendingFolders = pending)
    }
}
