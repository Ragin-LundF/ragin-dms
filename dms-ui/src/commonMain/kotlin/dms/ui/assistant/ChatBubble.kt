package dms.ui.assistant

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import dms.application.port.assistant.model.ChatRole
import dms.application.port.assistant.model.ChatTurnModel
import dms.ui.theme.Spacing

/** One message: the user's on the right, the AI's on the left; both can be selected and copied. */
@Composable
fun ChatBubble(turn: ChatTurnModel) {
    val mine = turn.role == ChatRole.USER
    val colors = MaterialTheme.colorScheme
    Box(
        modifier = Modifier.fillMaxWidth(),
        contentAlignment = if (mine) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        SelectionContainer {
            Text(
                text = turn.text,
                style = MaterialTheme.typography.bodyLarge,
                color = if (mine) colors.onPrimaryContainer else colors.onSurface,
                modifier = Modifier
                    .widthIn(max = 520.dp)
                    .clip(shape = MaterialTheme.shapes.large)
                    .background(color = if (mine) colors.primaryContainer else colors.surfaceVariant)
                    .padding(horizontal = Spacing.large, vertical = Spacing.medium),
            )
        }
    }
}
