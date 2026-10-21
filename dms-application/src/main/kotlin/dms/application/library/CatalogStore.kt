package dms.application.library

import dms.application.common.LibrarySnapshot
import dms.application.common.OperationOutcome
import dms.application.port.state.SettingsRepository
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogUpdate
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Holds the catalog of the open library and is the only writer of it.
 *
 * Every change is applied to the catalog as it is at commit time, under a lock, so two use cases
 * running at once cannot overwrite each other's changes. A change is published only after the primary
 * folder saved it.
 */
class CatalogStore(
    private val settings: SettingsRepository,
    private val replicator: Replicator,
) {

    private val mutex = Mutex()
    private val state = MutableStateFlow<Catalog?>(value = null)
    private var foreignFolders: Set<LibraryFolderId> = emptySet()

    /** The catalog of the open library, or `null` while none is open. */
    val catalog: StateFlow<Catalog?> = state.asStateFlow()

    /**
     * The folders and the catalog, or `null` while no library is open. Folders that hold another
     * library are left out, so no use case ever writes to them.
     */
    suspend fun current(): LibrarySnapshot? {
        val configured = settings.loadLibraryFolders() ?: return null
        val catalog = state.value ?: return null
        // The primary's library id always wins, so a foreign folder is never the primary.
        val folders = LibraryFolders(folders = configured.folders.filterNot { it.id in foreignFolders })
        return LibrarySnapshot(folders = folders, catalog = catalog)
    }

    /**
     * Makes [catalog] the open catalog without saving it; used when a library is opened.
     * [foreign] are folders holding another library, excluded until the library is opened again.
     */
    fun publish(catalog: Catalog, foreign: Set<LibraryFolderId> = emptySet()) {
        foreignFolders = foreign
        state.value = catalog
    }

    /** Applies [edit] to the current catalog and saves the result to every library folder. */
    suspend fun commit(edit: (Catalog) -> CatalogUpdate): OperationOutcome {
        return mutex.withLock {
            val snapshot = current() ?: return@withLock OperationOutcome.LibraryNotOpen
            when (val update = edit(snapshot.catalog)) {
                is CatalogUpdate.Rejected -> OperationOutcome.Rejected(reason = update.reason)
                is CatalogUpdate.Applied -> save(snapshot = snapshot, catalog = update.catalog)
            }
        }
    }

    private suspend fun save(snapshot: LibrarySnapshot, catalog: Catalog): OperationOutcome {
        return when (val replication = replicator.saveCatalog(folders = snapshot.folders, catalog = catalog)) {
            is Replication.PrimaryFailed -> OperationOutcome.StorageFailed(failure = replication.failure)
            is Replication.Done -> {
                state.value = catalog
                OperationOutcome.Done(catalog = catalog, pendingFolders = replication.pendingFolders)
            }
        }
    }
}
