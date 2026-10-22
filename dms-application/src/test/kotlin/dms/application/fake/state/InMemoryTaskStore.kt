package dms.application.fake.state

import dms.application.port.state.TaskStore
import dms.domain.catalog.CatalogMergeConflict
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.task.model.TaskKeyModel

class InMemoryTaskStore : TaskStore {

    private val suggestions = mutableMapOf<DocumentIdModel, ClassificationSuggestionModel>()
    private var conflicts = listOf<CatalogMergeConflict>()
    private val dismissed = mutableSetOf<TaskKeyModel>()

    override suspend fun saveSuggestion(suggestion: ClassificationSuggestionModel) {
        suggestions[suggestion.documentId] = suggestion
    }

    override suspend fun suggestions(): List<ClassificationSuggestionModel> {
        return suggestions.values.toList()
    }

    override suspend fun removeSuggestion(documentId: DocumentIdModel) {
        suggestions.remove(key = documentId)
    }

    override suspend fun replaceMergeConflicts(conflicts: List<CatalogMergeConflict>) {
        this.conflicts = conflicts
    }

    override suspend fun mergeConflicts(): List<CatalogMergeConflict> {
        return conflicts
    }

    override suspend fun dismiss(key: TaskKeyModel) {
        dismissed += key
    }

    override suspend fun dismissedKeys(): Set<TaskKeyModel> {
        return dismissed.toSet()
    }
}
