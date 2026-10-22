package dms.application.port.state

import dms.domain.catalog.CatalogMergeConflict
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.task.model.TaskKeyModel

/**
 * Local state behind the dashboard that is not part of the catalog: pending AI suggestions, the
 * conflicts of the last catalog merge, and tasks the user dismissed.
 */
interface TaskStore {

    suspend fun saveSuggestion(suggestion: ClassificationSuggestionModel)

    suspend fun suggestions(): List<ClassificationSuggestionModel>

    suspend fun removeSuggestion(documentId: DocumentIdModel)

    suspend fun replaceMergeConflicts(conflicts: List<CatalogMergeConflict>)

    suspend fun mergeConflicts(): List<CatalogMergeConflict>

    suspend fun dismiss(key: TaskKeyModel)

    suspend fun dismissedKeys(): Set<TaskKeyModel>
}
