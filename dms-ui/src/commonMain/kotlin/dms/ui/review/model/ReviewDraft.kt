package dms.ui.review.model

/**
 * The suggestion as the user is editing it. Dates are `YYYY-MM-DD`, the category is a name or a
 * path like `Parent / Child`, and tags are separated by commas.
 */
data class ReviewDraft(
    val title: ReviewField,
    val documentDate: ReviewField,
    val correspondent: ReviewField,
    val category: ReviewField,
    val tags: ReviewField,
    val fileName: ReviewField,
)
