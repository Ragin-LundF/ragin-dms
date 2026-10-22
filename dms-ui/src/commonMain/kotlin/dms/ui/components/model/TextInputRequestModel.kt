package dms.ui.components.model

/** A request for one line of text, such as a new name. */
data class TextInputRequestModel(
    val title: String,
    val label: String,
    val initialValue: String,
    val confirmLabel: String,
)
