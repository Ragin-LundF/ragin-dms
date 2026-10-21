package dms.ui.dashboard.task

import dms.domain.task.OpenTask
import dms.ui.dashboard.model.TaskGroup

/** Which group the dashboard lists a task in. A new file that copies a document is a duplicate. */
object TaskGroups {

    fun of(task: OpenTask, context: TaskContext): TaskGroup {
        return when (task) {
            is OpenTask.ReviewSuggestion -> TaskGroup.SUGGESTIONS
            is OpenTask.AdoptNewFile ->
                if (context.originalOf(change = task.change) == null) TaskGroup.NEW_FILES else TaskGroup.DUPLICATES

            is OpenTask.RestoreOrDelete -> TaskGroup.MISSING
            is OpenTask.ReviewModification -> TaskGroup.MODIFIED
            is OpenTask.ConfirmMove -> TaskGroup.MOVED
            is OpenTask.ResolveDuplicates -> TaskGroup.DUPLICATES
            is OpenTask.ResolveContentConflict -> TaskGroup.CONFLICTS
            is OpenTask.ReviewCatalogMerge -> TaskGroup.CATALOG_MERGE
            is OpenTask.RetryFailedOperations -> TaskGroup.FAILED_OPERATIONS
        }
    }
}
