package dms.ui.library.search

import dms.application.search.model.SearchOutcome
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.service.AppServices
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * Searches the text of the documents along with their names, while the user [enable]s it. It runs
 * in the background once the user stops typing, so the list shows the matches by name at once and
 * adds the matches in the text when they are found. It only reads, so it never waits for a busy
 * library.
 */
class ContentSearch(
    private val services: AppServices,
    queries: Flow<String>,
    scope: CoroutineScope,
) {

    private val enabled = MutableStateFlow(value = true)
    private val mutableState = MutableStateFlow(value = ContentSearchStateModel())

    val state: StateFlow<ContentSearchStateModel> = mutableState.asStateFlow()

    init {
        scope.launch {
            combine(
                flow = queries.map { it.trim() }.distinctUntilChanged(),
                flow2 = enabled,
                flow3 = services.library.catalog,
            ) { query, on, _ -> query to on }
                .collectLatest { (query, on) -> search(query = query, on = on) }
        }
    }

    fun enable(on: Boolean) {
        enabled.value = on
    }

    private suspend fun search(query: String, on: Boolean) {
        if (!on || query.isEmpty()) {
            mutableState.value = ContentSearchStateModel(enabled = on)
            return
        }
        mutableState.update { it.copy(enabled = true, searching = true) }
        delay(timeMillis = TYPING_PAUSE_MILLIS)
        val matches = when (val found = services.library.searchContents(query = query)) {
            is SearchOutcome.Found -> found.documentIds
            SearchOutcome.LibraryNotOpen -> emptySet()
        }
        mutableState.value = ContentSearchStateModel(enabled = true, matches = matches)
    }

    private companion object {
        const val TYPING_PAUSE_MILLIS = 300L
    }
}
