package dms.ui.assistant

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Send
import androidx.compose.material.icons.rounded.Stop
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import dms.ui.assistant.model.ChatState
import dms.ui.theme.Spacing

/** The question being typed. Enter sends it, Shift+Enter starts a new line; while the AI answers, Stop. */
@Composable
fun ChatInput(state: ChatState, controller: ChatController) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        OutlinedTextField(
            value = state.input,
            onValueChange = controller::type,
            placeholder = { Text(text = "Ask about this document") },
            maxLines = 4,
            modifier = Modifier
                .weight(weight = 1f)
                .onPreviewKeyEvent { event ->
                    val send = event.type == KeyEventType.KeyDown && event.key == Key.Enter && !event.isShiftPressed
                    if (send) {
                        controller.send()
                    }
                    send
                },
        )
        if (state.waiting) {
            FilledIconButton(onClick = controller::stop) {
                Icon(imageVector = Icons.Rounded.Stop, contentDescription = "Stop")
            }
        } else {
            FilledIconButton(onClick = { controller.send() }, enabled = state.input.isNotBlank()) {
                Icon(imageVector = Icons.AutoMirrored.Rounded.Send, contentDescription = "Send")
            }
        }
    }
}
