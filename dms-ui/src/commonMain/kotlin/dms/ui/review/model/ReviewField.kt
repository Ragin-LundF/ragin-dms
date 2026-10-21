package dms.ui.review.model

/** One suggested value: whether the user takes it, and the value, which the user may edit. */
data class ReviewField(
    val accepted: Boolean,
    val value: String,
)
