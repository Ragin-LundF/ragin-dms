package dms.ui.assistant

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Summarize
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.domain.document.Document
import dms.ui.theme.Spacing

/** Shows the AI's summary of a document while it is written; the text can be selected and copied. */
@Composable
fun SummaryDialog(controller: SummaryController, onAsk: (Document) -> Unit) {
    val summary by controller.state.collectAsState()
    val state = summary ?: return
    AlertDialog(
        onDismissRequest = controller::close,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 620.dp),
        icon = { Icon(imageVector = Icons.Rounded.Summarize, contentDescription = null) },
        title = { Text(text = "Summary of ${state.document.path.fileName}") },
        text = {
            Column(
                modifier = Modifier.heightIn(max = 480.dp).verticalScroll(state = rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
            ) {
                if (state.working) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
                    ) {
                        CircularProgressIndicator(modifier = Modifier.size(size = 20.dp), strokeWidth = 2.dp)
                        Text(text = "The AI is reading the document…")
                    }
                }
                state.text?.let { text ->
                    SelectionContainer { Text(text = text, style = MaterialTheme.typography.bodyLarge) }
                    Text(
                        text = "Written by ${state.model}. Check important facts in the document.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                state.problem?.let { problem ->
                    Text(text = problem, color = MaterialTheme.colorScheme.error)
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = {
                    controller.close()
                    onAsk(state.document)
                },
            ) { Text(text = "Ask a question…") }
        },
        dismissButton = {
            TextButton(onClick = controller::close) { Text(text = "Close") }
        },
    )
}
