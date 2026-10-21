package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.StorageResult
import dms.domain.change.DetectedChange

/**
 * Moves a file the catalog does not know to the system trash in every folder that has it, because it
 * is a copy of a catalogued document. Safe by design: the catalog must know a document with the same
 * size and SHA-256, and in each folder the two files are compared byte by byte right before the copy
 * goes. A folder where they are not identical, or where the document itself is missing, keeps the
 * file. Only with the user's confirmation; the catalog does not change.
 */
class TrashDuplicateFile(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(change: DetectedChange.NewFile, confirmed: Boolean): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val contentHash = change.contentHash ?: return OperationOutcome.ScanOutdated
        if (library.catalog.findByPath(path = change.path) != null) {
            return OperationOutcome.ScanOutdated
        }
        val original = library.catalog.findByContentHash(contentHash = contentHash)
            .firstOrNull { it.size == change.size } ?: return OperationOutcome.NotFound
        val trashed = library.folders.folders.filter { it.id in change.presentIn }.count { folder ->
            val same = storage.sameContent(folder = folder, first = original.path, second = change.path)
            (same as? StorageResult.Success)?.value == true &&
                storage.moveToTrash(folder = folder, path = change.path) is StorageResult.Success
        }
        if (trashed == 0) {
            return OperationOutcome.ScanOutdated
        }
        activity.record(kind = ActivityKind.DOCUMENT_TRASHED, subject = change.path.value)
        return OperationOutcome.Done(catalog = library.catalog, pendingFolders = emptySet())
    }
}
