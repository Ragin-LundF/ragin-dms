package dms.ui.shell.toolbar

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import dms.ui.components.ActionButtons
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.theme.Spacing

/**
 * A button with a menu of related actions beside it, such as Scan with Sync and Merge. Both wait while
 * [busyReason] names a running operation, and say so in their tooltip.
 */
@Composable
fun SplitButton(main: ActionButtonModel, menu: List<ActionButtonModel>, menuLabel: String, busyReason: String?) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
    ) {
        ActionButtons(buttons = listOf(main), enabled = busyReason == null, busyReason = busyReason)
        ActionMenu(
            actions = menu,
            contentDescription = menuLabel,
            enabled = busyReason == null,
            busyReason = busyReason,
            icon = Icons.Rounded.ArrowDropDown,
        )
    }
}
