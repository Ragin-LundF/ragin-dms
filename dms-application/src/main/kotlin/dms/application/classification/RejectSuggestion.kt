package dms.application.classification

import dms.application.port.state.TaskStore
import dms.domain.document.model.DocumentIdModel

/** Discards the pending suggestion of a document. The document stays as it is. */
class RejectSuggestion(private val tasks: TaskStore) {

    suspend operator fun invoke(documentId: DocumentIdModel) {
        tasks.removeSuggestion(documentId = documentId)
    }
}
