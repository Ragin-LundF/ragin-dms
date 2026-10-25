package dms.ui.library.state

import dms.application.folder.model.FoldersOutcome
import dms.domain.library.model.RelativePathModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update

/**
 * The folders that exist on disk in the primary library folder, also empty ones the catalog cannot know.
 * Read anew when a library opens or closes, after a scan, and after every folder action ([refresh]), not on
 * every edit, so the tree, the tiles and the pickers show every folder, also after a restart. Only reads.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class DiskFolders(private val services: AppServices, session: LibrarySession, scope: CoroutineScope) {

    private val refreshes = MutableStateFlow(value = 0)
    private var last: Set<RelativePathModel> = emptySet()

    val folders: StateFlow<Set<RelativePathModel>> = combine(
        flow = services.library.catalog.map { it == null }.distinctUntilChanged(),
        flow2 = session.state.map { it.lastScan to it.lastScanAt }.distinctUntilChanged(),
        flow3 = refreshes,
    ) { _, _, _ -> Unit }
        .mapLatest { read() }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = emptySet())

    /** Reads the folders again, after something created, moved or deleted one. */
    fun refresh() {
        refreshes.update { it + 1 }
    }

    private suspend fun read(): Set<RelativePathModel> {
        return when (val outcome = services.documents.readFolders()) {
            is FoldersOutcome.Read -> outcome.folders.also { last = it }
            FoldersOutcome.LibraryNotOpen -> emptySet<RelativePathModel>().also { last = it }
            // Unreadable for a moment, such as a primary offline: keep what was known.
            is FoldersOutcome.StorageFailed -> last
        }
    }
}
