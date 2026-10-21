package dms.application.document

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.library.CatalogStore
import dms.application.port.state.TaskStore
import dms.application.port.state.model.ActivityKind
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.document.DocumentId

/**
 * Moves a document to the system trash in every library folder and removes it from the catalog.
 * The only way the tool deletes anything, and only with the user's confirmation.
 */
class TrashDocument(
    private val store: CatalogStore,
    private val replicator: Replicator,
    private val tasks: TaskStore,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(documentId: DocumentId, confirmed: Boolean): OperationOutcome {
        if (!confirmed) {
            return OperationOutcome.NotConfirmed
        }
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return OperationOutcome.NotFound
        val replication = replicator.trash(folders = library.folders, path = document.path)
        if (replication is Replication.PrimaryFailed) {
            return OperationOutcome.StorageFailed(failure = replication.failure)
        }
        val pending = (replication as Replication.Done).pendingFolders
        val outcome = store.commit { it.removeDocument(documentId = documentId) }.withPendingFolders(folders = pending)
        if (outcome is OperationOutcome.Done) {
            tasks.removeSuggestion(documentId = documentId)
            activity.record(kind = ActivityKind.DOCUMENT_TRASHED, subject = document.path.value)
        }
        return outcome
    }
}
