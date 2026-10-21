package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.StorageResult
import dms.domain.change.DetectedChange

/**
 * Moves a file the catalog does not know to the system trash in every folder that has it: the
 * other answer to a file found on one side only, besides adopting it. Only with the user's
 * confirmation; the catalog does not change.
 */
class TrashNewFile(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(change: DetectedChange.NewFile, confirmed: Boolean): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        if (library.catalog.findByPath(path = change.path) != null) {
            return OperationOutcome.ScanOutdated
        }
        library.folders.folders.filter { it.id in change.presentIn }.forEach { folder ->
            val trashed = storage.moveToTrash(folder = folder, path = change.path)
            if (trashed is StorageResult.Failure) {
                return OperationOutcome.StorageFailed(failure = trashed)
            }
        }
        activity.record(kind = ActivityKind.DOCUMENT_TRASHED, subject = change.path.value)
        return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
    }
}
