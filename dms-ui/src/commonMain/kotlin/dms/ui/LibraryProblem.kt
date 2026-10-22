package dms.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CloudOff
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.StyledButton
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.EmptyState
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.LibraryStatus
import dms.ui.theme.Spacing

/** Why the library could not be opened, with the ways to fix it. */
@Composable
fun LibraryProblem(status: LibraryStatus, onRetry: () -> Unit, onOpenSettings: () -> Unit) {
    val text = when (status) {
        is LibraryStatus.PrimaryFailed -> {
            val cause = listOfNotNull(OutcomeMessages.failure(reason = status.reason), status.detail)
                .joinToString(separator = ": ")
            "The primary folder or its catalog cannot be read ($cause). " +
                "Connect the drive and try again, or make another folder the primary in the settings."
        }

        is LibraryStatus.Failed -> "The library could not be opened: ${status.detail ?: "unknown error"}"
        else -> "The library is not open."
    }
    EmptyState(
        icon = Icons.Rounded.CloudOff,
        title = "The library is not open",
        message = text,
        modifier = Modifier.fillMaxSize(),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            StyledButton(
                button = ActionButtonModel(
                    label = "Try again",
                    icon = Icons.Rounded.Refresh,
                    style = ButtonStyle.FILLED,
                    onClick = onRetry,
                ),
                enabled = true,
            )
            StyledButton(
                button = ActionButtonModel(label = "Settings", icon = Icons.Rounded.Settings, onClick = onOpenSettings),
                enabled = true,
            )
        }
    }
}
