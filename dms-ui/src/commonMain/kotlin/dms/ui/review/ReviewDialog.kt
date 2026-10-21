package dms.ui.review

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.domain.document.DocumentId
import dms.ui.theme.Spacing

/**
 * Shows the review [controller] has open: every suggested value, editable, approved field by field.
 * [onOpenDocument] opens the document itself, to check the values against it.
 */
@Composable
fun ReviewDialog(controller: ReviewController, onOpenDocument: (DocumentId) -> Unit) {
    val review by controller.state.collectAsState()
    val state = review ?: return
    AlertDialog(
        onDismissRequest = controller::close,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 680.dp),
        icon = { Icon(imageVector = Icons.Rounded.AutoAwesome, contentDescription = null) },
        title = { Text(text = "AI suggestion for ${state.documentName}") },
        text = { ReviewFields(state = state, controller = controller) },
        confirmButton = {
            Button(onClick = controller::approve) { Text(text = "Apply checked values") }
        },
        dismissButton = {
            TextButton(onClick = { onOpenDocument(state.suggestion.documentId) }) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.OpenInNew,
                    contentDescription = null,
                    modifier = Modifier.padding(end = Spacing.small),
                )
                Text(text = "Open document")
            }
            TextButton(onClick = controller::reject) { Text(text = "Reject suggestion") }
            TextButton(onClick = controller::close) { Text(text = "Later") }
        },
    )
}
