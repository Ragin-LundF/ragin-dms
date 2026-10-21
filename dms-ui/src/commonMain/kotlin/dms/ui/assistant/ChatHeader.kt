package dms.ui.assistant

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.DeleteSweep
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.assistant.model.ChatState

/** The title of a conversation, with Start over and Close. */
@Composable
fun ChatHeader(state: ChatState, controller: ChatController) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(text = "Ask AI", style = MaterialTheme.typography.titleLarge)
            Text(
                text = "About ${state.document.path.fileName}. Answers come from the document; " +
                    "check what matters.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        IconButton(onClick = controller::clear, enabled = state.turns.isNotEmpty()) {
            Icon(imageVector = Icons.Rounded.DeleteSweep, contentDescription = "Start over")
        }
        IconButton(onClick = controller::close) {
            Icon(imageVector = Icons.Rounded.Close, contentDescription = "Close")
        }
    }
}
