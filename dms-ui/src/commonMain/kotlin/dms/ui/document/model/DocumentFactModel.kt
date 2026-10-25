package dms.ui.document.model

/**
 * A custom field among the key facts: its [label] and [value], `null` without one, and for a deadline
 * a [chip] such as "in 11 days", [urgent] once the deadline is within its lead time or past.
 */
data class DocumentFactModel(
    val label: String,
    val value: String?,
    val chip: String? = null,
    val urgent: Boolean = false,
)
