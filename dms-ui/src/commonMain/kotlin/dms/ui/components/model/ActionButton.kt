package dms.ui.components.model

/**
 * A button computed by a view model. A button that cannot be used now names the reason in
 * [disabledReason], which the UI shows as its tooltip.
 */
data class ActionButton(
    val label: String,
    val disabledReason: String? = null,
    val onClick: () -> Unit,
)
