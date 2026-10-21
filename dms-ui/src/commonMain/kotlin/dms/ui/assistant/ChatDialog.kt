package dms.ui.assistant

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import dms.ui.theme.Spacing

/** A conversation with the AI about one document. Answers come from the document only. */
@Composable
fun ChatDialog(controller: ChatController) {
    val chat by controller.state.collectAsState()
    val state = chat ?: return
    val listState = rememberLazyListState()
    LaunchedEffect(state.turns.size, state.waiting) {
        val last = state.turns.size + if (state.waiting) 1 else 0
        if (last > 0) {
            listState.animateScrollToItem(index = last - 1)
        }
    }
    Dialog(onDismissRequest = controller::close, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Surface(
            shape = MaterialTheme.shapes.extraLarge,
            tonalElevation = 6.dp,
            modifier = Modifier.size(width = 760.dp, height = 680.dp),
        ) {
            Column(
                modifier = Modifier.padding(all = Spacing.xlarge),
                verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
            ) {
                ChatHeader(state = state, controller = controller)
                LazyColumn(
                    state = listState,
                    modifier = Modifier.weight(weight = 1f).fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
                ) {
                    if (state.turns.isEmpty() && !state.waiting) {
                        item { ChatStarters(onAsk = controller::send) }
                    }
                    items(items = state.turns) { turn -> ChatBubble(turn = turn) }
                    if (state.waiting) {
                        item { CircularProgressIndicator(modifier = Modifier.size(size = 24.dp), strokeWidth = 2.dp) }
                    }
                }
                state.problem?.let { problem ->
                    Text(text = problem, color = MaterialTheme.colorScheme.error)
                }
                ChatInput(state = state, controller = controller)
            }
        }
    }
}
