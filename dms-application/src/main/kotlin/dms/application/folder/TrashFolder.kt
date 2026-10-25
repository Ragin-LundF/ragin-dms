package dms.application.folder

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import dms.application.port.state.model.ActivityKind
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.CatalogUpdate
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Moves a folder with everything in it to the system trash in every library folder and removes its
 * documents from the catalog, so they can be restored from the trash. Only with the user's confirmation;
 * the library's top level cannot be deleted.
 */
class TrashFolder(
    private val store: CatalogStore,
    private val replicator: Replicator,
    private val tasks: TaskStore,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(path: RelativePathModel, confirmed: Boolean): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        if (path.isRoot) {
            return OperationOutcome.NotFound
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val replication = replicator.trash(folders = library.folders, path = path)
        if (replication is Replication.PrimaryFailed) {
            return OperationOutcome.StorageFailed(failure = replication.failure)
        }
        val pending = (replication as Replication.Done).pendingFolders
        // Taken at commit time, so a document added in the meantime is removed with its folder too.
        var removed = emptyList<DocumentIdModel>()
        val outcome = store.commit { catalog ->
            val inside = catalog.documents.values.filter { it.path.isInside(folder = path) }.map { it.id }
            removed = inside
            val start: CatalogUpdate = CatalogUpdate.Applied(catalog = catalog)
            inside.fold(initial = start) { update, id -> update.andThen { it.removeDocument(documentId = id) } }
        }.withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            removed.forEach { tasks.removeSuggestion(documentId = it) }
            activity.record(kind = ActivityKind.FOLDER_TRASHED, subject = path.value)
        }
        return outcome
    }
}
