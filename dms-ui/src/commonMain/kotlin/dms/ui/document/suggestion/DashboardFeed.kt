package dms.ui.document.suggestion

import dms.domain.task.OpenTask
import dms.ui.document.model.DashboardFeedModel
import dms.ui.session.LibrarySession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

/**
 * The pending suggestions and the recent activity for quick look, from the session's one dashboard, so
 * both always agree.
 */
class DashboardFeed(session: LibrarySession, scope: CoroutineScope) {

    val state: StateFlow<DashboardFeedModel> = session.dashboard
        .map { dashboard ->
            DashboardFeedModel(
                suggestions = dashboard?.tasks.orEmpty()
                    .filterIsInstance<OpenTask.ReviewSuggestion>()
                    .associate { it.suggestion.documentId to it.suggestion },
                activity = dashboard?.recentActivity.orEmpty(),
            )
        }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = DashboardFeedModel())
}
