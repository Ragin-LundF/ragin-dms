package dms.ui.shell.navigation

import dms.domain.catalog.model.CatalogModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.shell.navigation.model.NavigationCountsModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn

/** Counts what the navigation badges show, from the catalog and the session's one dashboard. Only reads. */
class NavigationCounts(services: AppServices, session: LibrarySession, scope: CoroutineScope) {

    val counts: StateFlow<NavigationCountsModel> = combine(
        flow = services.library.catalog,
        flow2 = session.dashboard,
    ) { catalog, dashboard ->
        NavigationCountsModel(inbox = inboxCount(catalog = catalog), tasks = dashboard?.tasks?.size ?: 0)
    }.stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = NavigationCountsModel())

    private fun inboxCount(catalog: CatalogModel?): Int {
        val inbox = catalog?.settings?.layout?.inbox?.takeIf { it.enabled } ?: return 0
        return catalog.documents.values.count { inbox.contains(candidate = it.path) }
    }
}
