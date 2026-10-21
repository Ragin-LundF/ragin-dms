package dms.ui.assistant

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import dms.ui.theme.Spacing

/** Questions to start with, for an empty conversation. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ChatStarters(onAsk: (String) -> Unit) {
    val questions = listOf(
        "What is this document about?",
        "Do I have to do anything, and by when?",
        "Which amounts are mentioned?",
        "Does this matter for my tax return?",
    )
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        questions.forEach { question ->
            AssistChip(onClick = { onAsk(question) }, label = { Text(text = question) })
        }
    }
}
