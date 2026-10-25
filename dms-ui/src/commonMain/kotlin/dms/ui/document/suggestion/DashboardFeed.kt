package dms.ui.document.suggestion

import dms.application.library.model.outcome.DashboardOutcome
import dms.domain.task.OpenTask
import dms.ui.document.model.DashboardFeedModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

/**
 * The pending suggestions and the recent activity for the inspector, from the same use case as the
 * dashboard, so both always agree. It starts when the inspector shows it first, and from then on is
 * rebuilt whenever the catalog changes or an operation ends.
 */
class DashboardFeed(services: AppServices, session: LibrarySession, scope: CoroutineScope) {

    val state: StateFlow<DashboardFeedModel> = combine(
        flow = session.state.map { it.lastScan to it.revision }.distinctUntilChanged(),
        flow2 = services.library.catalog,
    ) { (scan, _), _ -> scan }
        .map { scan ->
            when (val outcome = services.library.buildDashboard(scan = scan)) {
                is DashboardOutcome.Built -> DashboardFeedModel(
                    suggestions = outcome.dashboard.tasks
                        .filterIsInstance<OpenTask.ReviewSuggestion>()
                        .associate { it.suggestion.documentId to it.suggestion },
                    activity = outcome.dashboard.recentActivity,
                )

                DashboardOutcome.LibraryNotOpen -> DashboardFeedModel()
            }
        }
        .stateIn(scope = scope, started = SharingStarted.Lazily, initialValue = DashboardFeedModel())
}
