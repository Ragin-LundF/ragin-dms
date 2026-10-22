package dms.ui.dashboard.task

import dms.domain.catalog.CatalogMergeConflict
import dms.domain.change.DetectedChange
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.task.OpenTask
import dms.ui.components.FileSizes

/** The title and the detail line of every kind of task. */
object TaskTexts {

    fun title(task: OpenTask, context: TaskContext): String {
        return when (task) {
            is OpenTask.ReviewSuggestion -> "Suggestion for ${context.documentPath(id = task.suggestion.documentId)}"
            is OpenTask.AdoptNewFile -> newFileTitle(task = task, context = context)

            is OpenTask.RestoreOrDelete -> "Missing ${task.change.path}"
            is OpenTask.ReviewModification -> "Changed outside the tool: ${task.change.path}"
            is OpenTask.ConfirmMove ->
                "Other name in ${context.folderName(id = task.change.folderId)}: ${task.change.path}"

            is OpenTask.ResolveDuplicates -> "${task.documentIds.size} copies of the same file"
            is OpenTask.ResolveContentConflict -> "Different content: ${task.change.path}"
            is OpenTask.ReviewCatalogMerge -> mergeTitle(conflict = task.conflict)
            is OpenTask.RetryFailedOperations ->
                "${task.count} operation(s) waiting for ${context.folderName(id = task.folderId)}"

            is OpenTask.DeadlineDue -> {
                val label = context.fieldLabel(key = task.fieldKey)
                "$label on ${task.date}: ${context.documentPath(id = task.documentId)}"
            }

            is OpenTask.RetentionDue -> "Retention ended: ${context.documentPath(id = task.documentId)}"
            is OpenTask.RestructureSuggested -> "${task.fileCount} files without structure in ${task.folder}"
            is OpenTask.ReviewLearnedRule -> "Rule for ${task.proposal.correspondent}"
        }
    }

    fun detail(task: OpenTask, context: TaskContext): String {
        return when (task) {
            is OpenTask.ReviewSuggestion -> suggestionDetail(suggestion = task.suggestion, context = context)
            is OpenTask.AdoptNewFile -> newFileDetail(change = task.change, context = context)

            is OpenTask.RestoreOrDelete ->
                "Missing in ${context.folderNames(ids = task.change.missingIn)}; " +
                    "still in ${context.folderNames(ids = task.change.presentIn)}"

            is OpenTask.ReviewModification ->
                "The content in ${context.folderName(id = task.change.folderId)} differs from the catalog"

            is OpenTask.ConfirmMove -> "The catalog and the other folders call it ${task.change.previousPath}"
            is OpenTask.ResolveDuplicates ->
                "Same size and SHA-256 checksum. Keep one; the others go to the trash in every folder."

            is OpenTask.ResolveContentConflict -> {
                val folders = task.change.contentHashes.keys
                "${folders.size} versions in ${context.folderNames(ids = folders)}"
            }

            is OpenTask.ReviewCatalogMerge -> mergeDetail(conflict = task.conflict)
            is OpenTask.RetryFailedOperations -> "They are applied in order once the folder is reachable"
            is OpenTask.DeadlineDue -> "Dismiss once it is handled; a new date brings the task back"
            is OpenTask.RetentionDue ->
                "You chose to keep it until ${task.date}. Move it to the trash, or keep it and mark this as done."

            is OpenTask.RestructureSuggested -> "The restructure assistant can sort them into subfolders"
            is OpenTask.ReviewLearnedRule -> learnedRuleDetail(task = task, context = context)
        }
    }

    private fun newFileTitle(task: OpenTask.AdoptNewFile, context: TaskContext): String {
        if (context.originalOf(change = task.change) == null) {
            return "New file ${task.change.path}"
        }
        return "Copy of a document: ${task.change.path}"
    }

    private fun learnedRuleDetail(task: OpenTask.ReviewLearnedRule, context: TaskContext): String {
        val proposal = task.proposal
        val parts = listOfNotNull(
            proposal.categoryId?.let { "Category: ${context.categoryName(id = it)}" },
            proposal.tags.takeIf { it.isNotEmpty() }?.let { tags -> "Tags: ${tags.joinToString { it.value }}" },
        )
        return "From ${proposal.documentCount} approved documents · ${parts.joinToString(separator = " · ")}"
    }

    private fun newFileDetail(change: DetectedChange.NewFile, context: TaskContext): String {
        val where = "In ${context.folderNames(ids = change.presentIn)}, ${FileSizes.format(bytes = change.size)}"
        val original = context.originalOf(change = change) ?: return where
        return "$where; same size and checksum as ${original.path}"
    }

    private fun suggestionDetail(suggestion: ClassificationSuggestionModel, context: TaskContext): String {
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
            is CatalogMergeConflict.FieldDefinitionDiffers -> "Catalog copies differed for field ${conflict.kept.label}"
            is CatalogMergeConflict.SavedSearchDiffers ->
                "Catalog copies differed for smart folder ${conflict.kept.name}"

            is CatalogMergeConflict.RuleDiffers -> "Catalog copies differed for rule ${conflict.kept.name}"
        }
    }

    private fun mergeDetail(conflict: CatalogMergeConflict): String {
        return when (conflict) {
            is CatalogMergeConflict.DocumentDiffers -> "The newer entry was kept"
            is CatalogMergeConflict.PathClash -> "The newer entry was kept; the other one was dropped from the catalog"
            is CatalogMergeConflict.CategoryDiffers ->
                "Kept \"${conflict.kept.name}\", dropped \"${conflict.discarded.name}\""

            is CatalogMergeConflict.FieldDefinitionDiffers,
            is CatalogMergeConflict.SavedSearchDiffers,
            is CatalogMergeConflict.RuleDiffers,
            -> "The newer version was kept"
        }
    }
}
