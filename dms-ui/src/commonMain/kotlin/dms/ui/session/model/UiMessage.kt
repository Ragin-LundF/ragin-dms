package dms.ui.session.model

/** A short message for the status line, such as the result of an action. */
data class UiMessage(
    val text: String,
    val isError: Boolean = false,
)
