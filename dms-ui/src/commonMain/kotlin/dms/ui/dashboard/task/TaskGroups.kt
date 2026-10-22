package dms.ui.dashboard.task

import dms.domain.task.OpenTask
import dms.ui.dashboard.model.TaskGroup

/** Which group the dashboard lists a task in. A new file that copies a document is a duplicate. */
object TaskGroups {

    fun of(task: OpenTask, context: TaskContext): TaskGroup {
        return when (task) {
            is OpenTask.ReviewSuggestion -> TaskGroup.SUGGESTIONS
            is OpenTask.AdoptNewFile -> newFileGroup(task = task, context = context)

            is OpenTask.RestoreOrDelete -> TaskGroup.MISSING
            is OpenTask.ReviewModification -> TaskGroup.MODIFIED
            is OpenTask.ConfirmMove -> TaskGroup.MOVED
            is OpenTask.ResolveDuplicates -> TaskGroup.DUPLICATES
            is OpenTask.ResolveContentConflict -> TaskGroup.CONFLICTS
            is OpenTask.ReviewCatalogMerge -> TaskGroup.CATALOG_MERGE
            is OpenTask.RetryFailedOperations -> TaskGroup.FAILED_OPERATIONS
            is OpenTask.DeadlineDue -> TaskGroup.DEADLINES
            is OpenTask.RetentionDue -> TaskGroup.RETENTION
            is OpenTask.RestructureSuggested -> TaskGroup.RESTRUCTURE
            is OpenTask.ReviewLearnedRule -> TaskGroup.LEARNED_RULES
        }
    }

    private fun newFileGroup(task: OpenTask.AdoptNewFile, context: TaskContext): TaskGroup {
        return if (context.originalOf(change = task.change) == null) TaskGroup.NEW_FILES else TaskGroup.DUPLICATES
    }
}
