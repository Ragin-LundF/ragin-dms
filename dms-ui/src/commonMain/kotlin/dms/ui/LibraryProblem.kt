package dms.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.LibraryStatus
import dms.ui.theme.Spacing

/** Why the library could not be opened, with the ways to fix it. */
@Composable
fun LibraryProblem(status: LibraryStatus, onRetry: () -> Unit, onOpenSettings: () -> Unit) {
    val text = when (status) {
        is LibraryStatus.PrimaryFailed ->
            "The primary folder or its catalog cannot be read (${OutcomeMessages.failure(reason = status.reason)}). " +
                "Connect the drive and try again, or make another folder the primary in the settings."

        is LibraryStatus.Failed -> "The library could not be opened: ${status.detail ?: "unknown error"}"
        else -> "The library is not open."
    }
    Column(
        modifier = Modifier.fillMaxSize().padding(all = Spacing.xlarge),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Text(text = "The library is not open", style = MaterialTheme.typography.headlineSmall)
        Text(text = text, modifier = Modifier.widthIn(max = 640.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            Button(onClick = onRetry) { Text(text = "Try again") }
            OutlinedButton(onClick = onOpenSettings) { Text(text = "Settings") }
        }
    }
}
