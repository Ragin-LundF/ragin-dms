package dms.ui.ask

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.Send
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material.icons.rounded.Description
import androidx.compose.material.icons.rounded.QuestionAnswer
import androidx.compose.material.icons.rounded.Sync
import androidx.compose.material3.AssistChip
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.ui.ask.model.AskEntryModel
import dms.ui.ask.model.AskUiStateModel
import dms.ui.components.ActionButtons
import dms.ui.components.layout.ScreenHeader
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.EmptyState
import dms.ui.session.OutcomeMessages
import dms.ui.theme.Spacing

/**
 * Ask your library: a question, the answer with the documents it comes from as chips that open them,
 * and the scope to ask in. The index behind it is updated on demand.
 */
@Composable
fun AskScreen(viewModel: AskViewModel, onOpenDocument: (DocumentIdModel) -> Unit) {
    val state by viewModel.state.collectAsState()
    var question by remember { mutableStateOf(value = "") }
    val aiBusyReason = state.aiBusy?.let { OutcomeMessages.waitFor(operation = it) }
    Column(
        modifier = Modifier.fillMaxSize().padding(all = Spacing.screen),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        ScreenHeader(title = "Ask", subtitle = "Questions answered from your documents, with their sources") {
            ActionButtons(
                buttons = listOf(
                    ActionButtonModel(
                        label = "Update index",
                        icon = Icons.Rounded.Sync,
                        onClick = viewModel::updateIndex,
                    ),
                ),
                enabled = aiBusyReason == null,
                busyReason = aiBusyReason,
            )
        }
        Box(modifier = Modifier.weight(weight = 1f).fillMaxWidth()) {
            if (state.entries.isEmpty()) {
                EmptyState(
                    icon = Icons.Rounded.QuestionAnswer,
                    title = "Ask about your documents",
                    message = "For example: \"When does my car insurance end?\" or " +
                        "\"How much did I pay for electricity in " +
                        "2024?\" Update the index once, so questions find documents by meaning, not only by words.",
                    modifier = Modifier.fillMaxWidth(),
                )
            } else {
                LazyColumn(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
                    items(items = state.entries) { entry -> EntryCard(entry = entry, onOpenDocument = onOpenDocument) }
                }
            }
        }
        QuestionRow(
            state = state,
            question = question,
            onQuestion = { question = it },
            onSelectScope = viewModel::selectScope,
            onAsk = {
                viewModel.ask(question = question)
                question = ""
            },
        )
    }
}

@Composable
private fun QuestionRow(
    state: AskUiStateModel,
    question: String,
    onQuestion: (String) -> Unit,
    onSelectScope: (Int) -> Unit,
    onAsk: () -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        ScopeMenu(state = state, onSelect = onSelectScope)
        OutlinedTextField(
            value = question,
            onValueChange = onQuestion,
            label = { Text(text = "Your question") },
            singleLine = true,
            modifier = Modifier.weight(weight = 1f),
        )
        IconButton(onClick = onAsk, enabled = question.isNotBlank() && state.aiBusy == null) {
            Icon(imageVector = Icons.AutoMirrored.Rounded.Send, contentDescription = "Ask")
        }
    }
}

@Composable
private fun ScopeMenu(state: AskUiStateModel, onSelect: (Int) -> Unit) {
    var open by remember { mutableStateOf(value = false) }
    Box {
        OutlinedButton(onClick = { open = true }) {
            Text(
                text = state.scopes.getOrNull(index = state.scopeIndex) ?: AskUiStateModel.WHOLE_LIBRARY,
                maxLines = 1,
                softWrap = false,
            )
            Icon(imageVector = Icons.Rounded.ArrowDropDown, contentDescription = null)
        }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            state.scopes.forEachIndexed { index, scope ->
                DropdownMenuItem(
                    text = { Text(text = scope) },
                    onClick = {
                        onSelect(index)
                        open = false
                    },
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun EntryCard(entry: AskEntryModel, onOpenDocument: (DocumentIdModel) -> Unit) {
    AppCard(modifier = Modifier.fillMaxWidth()) {
        Text(text = entry.question, style = MaterialTheme.typography.titleSmall)
        when {
            entry.asking -> Text(text = "Searching your documents…", style = MaterialTheme.typography.bodyMedium)
            entry.answer != null -> Text(text = entry.answer, style = MaterialTheme.typography.bodyMedium)
        }
        entry.note?.let {
            Text(
                text = it,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            entry.citations.forEach { citation ->
                AssistChip(
                    onClick = { onOpenDocument(citation.documentId) },
                    label = { Text(text = "[${citation.number}] ${citation.title}", maxLines = 1, softWrap = false) },
                    leadingIcon = { Icon(imageVector = Icons.Rounded.Description, contentDescription = null) },
                )
            }
        }
    }
}
