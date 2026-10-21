package dms.ui.review.model

import dms.domain.classification.ClassificationSuggestion

/**
 * The open review: the [suggestion], the document's [current] values to compare with, the [draft]
 * being edited, the [categoryPaths] to pick from, and [problem] when the last approval was refused
 * before anything changed.
 */
data class ReviewState(
    val suggestion: ClassificationSuggestion,
    val documentName: String,
    val current: ReviewDraft,
    val draft: ReviewDraft,
    val categoryPaths: List<String> = emptyList(),
    val problem: String? = null,
)
