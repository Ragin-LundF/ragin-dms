package dms.ui.dashboard.task

import dms.domain.catalog.CatalogMergeConflict
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.task.OpenTask
import dms.ui.components.FileSizes

/** The title and the detail line of every kind of task. */
object TaskTexts {

    fun title(task: OpenTask, context: TaskContext): String {
        return when (task) {
            is OpenTask.ReviewSuggestion -> "Suggestion for ${context.documentPath(id = task.suggestion.documentId)}"
            is OpenTask.AdoptNewFile -> "New file ${task.change.path}"
            is OpenTask.RestoreOrDelete -> "Missing ${task.change.path}"
            is OpenTask.ReviewModification -> "Changed outside the tool: ${task.change.path}"
            is OpenTask.ConfirmMove -> "Moved outside the tool: ${task.change.path}"
            is OpenTask.ResolveContentConflict -> "Different content: ${task.change.path}"
            is OpenTask.ReviewCatalogMerge -> mergeTitle(conflict = task.conflict)
            is OpenTask.RetryFailedOperations ->
                "${task.count} operation(s) waiting for ${context.folderName(id = task.folderId)}"
        }
    }

    fun detail(task: OpenTask, context: TaskContext): String {
        return when (task) {
            is OpenTask.ReviewSuggestion -> suggestionDetail(suggestion = task.suggestion, context = context)
            is OpenTask.AdoptNewFile ->
                "In ${context.folderNames(ids = task.change.presentIn)}, ${FileSizes.format(bytes = task.change.size)}"

            is OpenTask.RestoreOrDelete ->
                "Missing in ${context.folderNames(ids = task.change.missingIn)}; " +
                    "still in ${context.folderNames(ids = task.change.presentIn)}"

            is OpenTask.ReviewModification ->
                "The content in ${context.folderName(id = task.change.folderId)} differs from the catalog"

            is OpenTask.ConfirmMove ->
                "In ${context.folderName(id = task.change.folderId)}, previously ${task.change.previousPath}"

            is OpenTask.ResolveContentConflict -> {
                val folders = task.change.contentHashes.keys
                "${folders.size} versions in ${context.folderNames(ids = folders)}"
            }

            is OpenTask.ReviewCatalogMerge -> mergeDetail(conflict = task.conflict)
            is OpenTask.RetryFailedOperations -> "They are applied in order once the folder is reachable"
        }
    }

    private fun suggestionDetail(suggestion: ClassificationSuggestion, context: TaskContext): String {
        val category = when (val suggested = suggestion.category) {
            is SuggestedCategory.Existing -> context.categoryName(id = suggested.categoryId)
            is SuggestedCategory.New -> "${suggested.name} (new)"
            null -> null
        }
        return listOfNotNull(
            suggestion.title?.let { "Title: $it" },
            category?.let { "Category: $it" },
            suggestion.tags.takeIf { it.isNotEmpty() }?.let { tags -> "Tags: ${tags.joinToString { it.name.value }}" },
            suggestion.fileName?.let { "Rename → $it" },
        ).joinToString(separator = " · ")
    }

    private fun mergeTitle(conflict: CatalogMergeConflict): String {
        return when (conflict) {
            is CatalogMergeConflict.DocumentDiffers -> "Catalog copies differed for ${conflict.kept.path}"
            is CatalogMergeConflict.PathClash -> "Two documents claimed ${conflict.kept.path}"
            is CatalogMergeConflict.CategoryDiffers -> "Catalog copies differed for category ${conflict.kept.name}"
        }
    }

    private fun mergeDetail(conflict: CatalogMergeConflict): String {
        return when (conflict) {
            is CatalogMergeConflict.DocumentDiffers -> "The newer entry was kept"
            is CatalogMergeConflict.PathClash -> "The newer entry was kept; the other one was dropped from the catalog"
            is CatalogMergeConflict.CategoryDiffers ->
                "Kept \"${conflict.kept.name}\", dropped \"${conflict.discarded.name}\""
        }
    }
}
