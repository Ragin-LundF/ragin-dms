package dms.ui.ask

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dms.application.ask.model.AskOutcome
import dms.domain.query.model.DocumentQueryModel
import dms.ui.ask.model.AskEntryModel
import dms.ui.ask.model.AskUiStateModel
import dms.ui.review.AiMessages
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.model.OperationLane
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.takeWhile
import kotlinx.coroutines.flow.update

/**
 * "Ask your library": questions answered from the documents, with their sources, in the whole
 * library or a smart folder. Building the index runs in the AI lane, beside the library's work.
 */
class AskViewModel(
    private val services: AppServices,
    private val session: LibrarySession,
) : ViewModel() {

    private val entries = MutableStateFlow<List<AskEntryModel>>(value = emptyList())
    private val scopeIndex = MutableStateFlow(value = 0)

    val state: StateFlow<AskUiStateModel> = combine(
        flow = entries,
        flow2 = scopeIndex,
        flow3 = services.library.catalog,
        flow4 = session.state,
    ) { asked, index, catalog, sessionState ->
        val searches = catalog?.settings?.savedSearches.orEmpty().map { "Smart folder: ${it.name}" }
        AskUiStateModel(
            entries = asked,
            scopes = listOf(AskUiStateModel.WHOLE_LIBRARY) + searches,
            scopeIndex = index.coerceIn(range = 0..searches.size),
            aiBusy = sessionState.aiBusy,
        )
    }.stateIn(scope = viewModelScope, started = SharingStarted.Eagerly, initialValue = AskUiStateModel())

    fun selectScope(index: Int) {
        scopeIndex.value = index
    }

    fun ask(question: String) {
        val text = question.trim().takeIf { it.isNotEmpty() } ?: return
        val scope = scopeQuery()
        entries.update { it + AskEntryModel(question = text, asking = true) }
        session.perform(label = "Asking the library", lane = OperationLane.AI, cancellable = true) {
            val entry = entryOf(question = text, outcome = services.ask.askLibrary(question = text, scope = scope))
            entries.update { asked -> asked.dropLast(n = 1) + entry }
        }.also { started ->
            if (!started) {
                val busy = AskEntryModel(
                    question = text,
                    note = "Wait until \"${session.state.value.aiBusy}\" finishes",
                )
                entries.update { asked -> asked.dropLast(n = 1) + busy }
            }
        }
    }

    fun clear() {
        entries.value = emptyList()
    }

    /** Embeds what is new or changed since the last run; the rest of the index stays. */
    fun updateIndex() {
        session.perform(label = "Updating the index", lane = OperationLane.AI, cancellable = true) {
            var indexed = 0
            var failure: String? = null
            services.ask.indexLibrary()
                .onEach { progress ->
                    session.progress(
                        progress = OperationProgressModel(
                            step = "Updating the index",
                            done = progress.done,
                            total = progress.total,
                        ),
                        lane = OperationLane.AI,
                    )
                    if (progress.indexed) {
                        indexed++
                    }
                    failure = progress.failure
                }
                .takeWhile { failure == null }
                .collect()
            val stopped = failure
            session.report(
                message = if (stopped == null) {
                    UiMessageModel(text = "The index is up to date; $indexed documents were read anew")
                } else {
                    UiMessageModel(text = "The index could not be built: $stopped", isError = true)
                },
            )
        }
    }

    private fun scopeQuery(): DocumentQueryModel {
        val index = state.value.scopeIndex
        if (index == 0) {
            return DocumentQueryModel.ALL
        }
        val searches = services.library.catalog.value?.settings?.savedSearches.orEmpty()
        return searches.getOrNull(index = index - 1)?.query ?: DocumentQueryModel.ALL
    }

    private fun entryOf(question: String, outcome: AskOutcome): AskEntryModel {
        return when (outcome) {
            is AskOutcome.Answered -> AskEntryModel(
                question = question,
                answer = outcome.text,
                citations = outcome.citations,
                note = WORDS_ONLY.takeUnless { outcome.usedVectors },
            )

            AskOutcome.NotFound -> AskEntryModel(
                question = question,
                note = "Nothing in your documents fits this question.",
            )
            AskOutcome.LibraryNotOpen -> AskEntryModel(question = question, note = "No library is open.")
            is AskOutcome.Failed -> AskEntryModel(
                question = question,
                note = AiMessages.failure(reason = outcome.reason),
            )
        }
    }

    private companion object {
        const val WORDS_ONLY = "Searched by words only; update the index to search by meaning"
    }
}
