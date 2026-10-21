package dms.application.fake.state

import dms.application.port.state.TaskStore
import dms.domain.catalog.CatalogMergeConflict
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.DocumentId
import dms.domain.task.TaskKey

class InMemoryTaskStore : TaskStore {

    private val suggestions = mutableMapOf<DocumentId, ClassificationSuggestion>()
    private var conflicts = listOf<CatalogMergeConflict>()
    private val dismissed = mutableSetOf<TaskKey>()

    override suspend fun saveSuggestion(suggestion: ClassificationSuggestion) {
        suggestions[suggestion.documentId] = suggestion
    }

    override suspend fun suggestions(): List<ClassificationSuggestion> {
        return suggestions.values.toList()
    }

    override suspend fun removeSuggestion(documentId: DocumentId) {
        suggestions.remove(key = documentId)
    }

    override suspend fun replaceMergeConflicts(conflicts: List<CatalogMergeConflict>) {
        this.conflicts = conflicts
    }

    override suspend fun mergeConflicts(): List<CatalogMergeConflict> {
        return conflicts
    }

    override suspend fun dismiss(key: TaskKey) {
        dismissed += key
    }

    override suspend fun dismissedKeys(): Set<TaskKey> {
        return dismissed.toSet()
    }
}
