package dms.ui.components.model

/**
 * A question before a destructive action. [affected] names every file or folder the action touches,
 * so the user sees what will happen before confirming.
 */
data class ConfirmationRequestModel(
    val title: String,
    val message: String,
    val affected: List<String>,
    val confirmLabel: String,
)
