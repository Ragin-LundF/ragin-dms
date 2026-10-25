package dms.ui.shell.navigation

import dms.application.library.model.outcome.DashboardOutcome
import dms.domain.catalog.model.CatalogModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.shell.navigation.model.NavigationCountsModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn

/**
 * Counts what the navigation badges show, anew whenever the catalog or the last scan changes. Only
 * reads, so it never waits for a busy library.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class NavigationCounts(private val services: AppServices, session: LibrarySession, scope: CoroutineScope) {

    val counts: StateFlow<NavigationCountsModel> = combine(
        flow = services.library.catalog,
        flow2 = session.state.map { it.lastScan to it.revision }.distinctUntilChanged(),
    ) { catalog, (scan, _) -> catalog to scan }
        .mapLatest { (catalog, scan) ->
            val tasks = when (val outcome = services.library.buildDashboard(scan = scan)) {
                is DashboardOutcome.Built -> outcome.dashboard.tasks.size
                DashboardOutcome.LibraryNotOpen -> 0
            }
            NavigationCountsModel(inbox = inboxCount(catalog = catalog), tasks = tasks)
        }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = NavigationCountsModel())

    private fun inboxCount(catalog: CatalogModel?): Int {
        val inbox = catalog?.settings?.layout?.inbox?.takeIf { it.enabled } ?: return 0
        return catalog.documents.values.count { inbox.contains(candidate = it.path) }
    }
}
