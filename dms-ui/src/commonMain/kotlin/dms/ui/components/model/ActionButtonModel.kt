package dms.ui.components.model

import androidx.compose.ui.graphics.vector.ImageVector

/**
 * A button computed by a view model. A button that cannot be used now names the reason in
 * [disabledReason], which the UI shows as its tooltip. [shortcut] is shown beside it in a menu.
 */
data class ActionButtonModel(
    val label: String,
    val icon: ImageVector? = null,
    val style: ButtonStyle = ButtonStyle.OUTLINED,
    val disabledReason: String? = null,
    val shortcut: String? = null,
    val onClick: () -> Unit,
)
