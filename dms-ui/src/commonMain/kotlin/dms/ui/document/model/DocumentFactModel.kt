package dms.ui.document.model

/**
 * One line of the inspector's details: a [label] and its [value], and for a deadline a [chip] such as
 * "in 11 days", [urgent] once the deadline is within its lead time or past.
 */
data class DocumentFactModel(
    val label: String,
    val value: String,
    val chip: String? = null,
    val urgent: Boolean = false,
)
