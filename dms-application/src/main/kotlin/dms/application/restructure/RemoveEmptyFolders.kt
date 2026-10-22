package dms.application.restructure

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.StorageResult
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Moves folders a restructure left without documents to the trash in every library folder, after the
 * user confirmed. A folder that holds a document again, or any visible file, is kept.
 */
class RemoveEmptyFolders(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(folders: List<RelativePathModel>, confirmed: Boolean): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val pending = mutableSetOf<LibraryFolderIdModel>()
        // Deepest first, so a parent is only checked after its emptied children are gone.
        folders.sortedByDescending { it.segments.size }.forEach { folder ->
            val catalogued = library.catalog.documents.values.any { it.path.isInside(folder = folder) }
            val listed = storage.listNames(folder = library.folders.primary, path = folder)
            val empty = listed is StorageResult.Success && listed.value.all { it.startsWith(prefix = ".") }
            if (folder.isRoot || catalogued || !empty) {
                return@forEach
            }
            when (val trashed = replicator.trash(folders = library.folders, path = folder)) {
                is Replication.PrimaryFailed -> return OperationOutcome.StorageFailed(failure = trashed.failure)
                is Replication.Done -> {
                    pending += trashed.pendingFolders
                    activity.record(kind = ActivityKind.DOCUMENT_TRASHED, subject = folder.value)
                }
            }
        }
        return OperationOutcome.Done(catalog = library.catalog, pendingFolders = pending)
    }
}
