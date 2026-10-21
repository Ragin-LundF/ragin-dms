package dms.application.port.state

import dms.domain.catalog.CatalogMergeConflict
import dms.domain.classification.ClassificationSuggestion
import dms.domain.document.DocumentId
import dms.domain.task.TaskKey

/**
 * Local state behind the dashboard that is not part of the catalog: pending AI suggestions, the
 * conflicts of the last catalog merge, and tasks the user dismissed.
 */
interface TaskStore {

    suspend fun saveSuggestion(suggestion: ClassificationSuggestion)

    suspend fun suggestions(): List<ClassificationSuggestion>

    suspend fun removeSuggestion(documentId: DocumentId)

    suspend fun replaceMergeConflicts(conflicts: List<CatalogMergeConflict>)

    suspend fun mergeConflicts(): List<CatalogMergeConflict>

    suspend fun dismiss(key: TaskKey)

    suspend fun dismissedKeys(): Set<TaskKey>
}
