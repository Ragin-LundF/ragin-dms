package dms.ui.review.model

import dms.domain.field.model.FieldKeyModel

/**
 * The suggestion as the user is editing it. Dates are `YYYY-MM-DD`, the category is a name or a
 * path like `Parent / Child`, tags are separated by commas, and the target folder is a library path
 * like `Home/Power`.
 */
data class ReviewDraftModel(
    val title: ReviewFieldModel,
    val documentDate: ReviewFieldModel,
    val correspondent: ReviewFieldModel,
    val category: ReviewFieldModel,
    val tags: ReviewFieldModel,
    val fileName: ReviewFieldModel,
    val targetFolder: ReviewFieldModel = ReviewFieldModel(accepted = false, value = ""),
    val fields: Map<FieldKeyModel, ReviewFieldModel> = emptyMap(),
)
