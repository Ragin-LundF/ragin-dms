package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.DocumentDates
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.port.system.Clock
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.domain.change.DetectedChange
import dms.domain.document.Document
import dms.domain.document.DocumentId

/**
 * Catalogs a file the scan found but the catalog does not know, and copies it into every folder that
 * lacks it: into the primary from a folder that has it, then from the primary into the sync folders.
 * The file keeps its name and place.
 */
class AdoptNewFile(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val dates: DocumentDates,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
    private val clock: Clock,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(change: DetectedChange.NewFile): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val contentHash = change.contentHash ?: return OperationOutcome.ScanOutdated
        val primary = library.folders.primary
        // Where the file was found first: there its creation date is still the original one.
        val source = library.folders.folders.firstOrNull { it.id in change.presentIn }
            ?: return OperationOutcome.ScanOutdated
        val metadata = dates.metadataFor(folder = source, path = change.path)
        if (primary.id !in change.presentIn) {
            val copied = storage.copyFile(from = source, to = primary, path = change.path, replaceExisting = false)
            if (copied is StorageResult.Failure) {
                return OperationOutcome.StorageFailed(failure = copied)
            }
        }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders.filter { it.id !in change.presentIn },
            operation = FolderOperation.CopyFromPrimary(path = change.path),
        )
        val document = Document(
            id = DocumentId(value = ids.newId()),
            path = change.path,
            contentHash = contentHash,
            size = change.size,
            addedAt = clock.now(),
            metadata = metadata,
        )
        val outcome = store.commit { it.addDocument(document = document) }.withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            activity.record(kind = ActivityKind.DOCUMENT_ADDED, subject = change.path.value)
        }
        return outcome
    }
}
