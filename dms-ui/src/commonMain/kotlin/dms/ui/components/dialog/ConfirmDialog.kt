package dms.ui.components.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.model.ConfirmationRequest
import dms.ui.theme.Spacing

/** Asks before a destructive action and lists everything it affects. */
@Composable
fun ConfirmDialog(request: ConfirmationRequest, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(text = request.title) },
        text = {
            Column {
                Text(text = request.message)
                Column(
                    modifier = Modifier
                        .heightIn(max = 240.dp)
                        .verticalScroll(state = rememberScrollState()),
                ) {
                    request.affected.forEach { entry ->
                        Text(
                            text = "• $entry",
                            style = MaterialTheme.typography.bodySmall,
                            maxLines = 1,
                            softWrap = false,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.heightIn(min = Spacing.large),
                        )
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onConfirm) { Text(text = request.confirmLabel) }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text(text = "Cancel") }
        },
    )
}
