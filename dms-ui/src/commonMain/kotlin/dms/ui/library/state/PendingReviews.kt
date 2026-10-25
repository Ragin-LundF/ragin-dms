package dms.ui.library.state

import dms.domain.document.model.DocumentIdModel
import dms.domain.task.OpenTask
import dms.ui.session.LibrarySession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

/**
 * The documents with an AI suggestion that waits for review, for "Needs review". They come from the
 * session's one dashboard, so the sidebar, the dashboard and quick look always agree.
 */
class PendingReviews(session: LibrarySession, scope: CoroutineScope) {

    val ids: StateFlow<Set<DocumentIdModel>> = session.dashboard
        .map { dashboard ->
            dashboard?.tasks.orEmpty()
                .filterIsInstance<OpenTask.ReviewSuggestion>()
                .map { it.suggestion.documentId }
                .toSet()
        }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = emptySet())
}
