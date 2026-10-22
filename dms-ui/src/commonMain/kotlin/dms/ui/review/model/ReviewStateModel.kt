package dms.ui.review.model

import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.field.model.FieldKeyModel

/**
 * The open review: the [suggestion], the document's [current] values to compare with, the [draft]
 * being edited, the [categoryPaths] and [folderPaths] to pick from, and [problem] when the last
 * approval was refused before anything changed.
 */
data class ReviewStateModel(
    val suggestion: ClassificationSuggestionModel,
    val documentName: String,
    val current: ReviewDraftModel,
    val draft: ReviewDraftModel,
    val categoryPaths: List<String> = emptyList(),
    val folderPaths: List<String> = emptyList(),
    val fieldLabels: Map<FieldKeyModel, String> = emptyMap(),
    val problem: String? = null,
)
