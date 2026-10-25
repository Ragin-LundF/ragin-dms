package dms.application.library

import dms.application.common.OperationOutcome
import dms.application.common.model.LibrarySnapshotModel
import dms.application.port.state.SettingsRepository
import dms.application.port.storage.CatalogRepository
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.storage.model.StorageFailure
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.CatalogMerger
import dms.domain.catalog.CatalogUpdate
import dms.domain.catalog.model.CatalogModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFoldersModel
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
 *
 * When another device saved a newer catalog into the primary in the meantime, the primary refuses the
 * save. The store then merges that catalog into its own, publishes the merge, applies the edit once more
 * on top and saves again.
 */
class CatalogStore(
    private val settings: SettingsRepository,
    private val catalogs: CatalogRepository,
    private val replicator: Replicator,
) {

    private val mutex = Mutex()
    private val state = MutableStateFlow<CatalogModel?>(value = null)
    private var foreignFolders: Set<LibraryFolderIdModel> = emptySet()

    /** The catalog of the open library, or `null` while none is open. */
    val catalog: StateFlow<CatalogModel?> = state.asStateFlow()

    /**
     * The folders and the catalog, or `null` while no library is open. Folders that hold another
     * library are left out, so no use case ever writes to them.
     */
    suspend fun current(): LibrarySnapshotModel? {
        val configured = settings.loadLibraryFolders() ?: return null
        val catalog = state.value ?: return null
        // The primary's library id always wins, so a foreign folder is never the primary.
        val folders = LibraryFoldersModel(folders = configured.folders.filterNot { it.id in foreignFolders })
        return LibrarySnapshotModel(folders = folders, catalog = catalog)
    }

    /**
     * Makes [catalog] the open catalog without saving it; used when a library is opened.
     * [foreign] are folders holding another library, excluded until the library is opened again.
     */
    fun publish(catalog: CatalogModel, foreign: Set<LibraryFolderIdModel> = emptySet()) {
        foreignFolders = foreign
        state.value = catalog
    }

    /**
     * Applies [edit] to the current catalog and saves the result to every library folder. [edit] may run
     * twice: a second time on the merged catalog when another device changed the primary meanwhile.
     */
    suspend fun commit(edit: (CatalogModel) -> CatalogUpdate): OperationOutcome {
        return mutex.withLock {
            val snapshot = current() ?: return@withLock OperationOutcome.LibraryNotOpen
            val outcome = apply(snapshot = snapshot, edit = edit)
            if (!outcome.isCatalogChanged()) {
                return@withLock outcome
            }
            val merged = mergeWithPrimary(snapshot = snapshot) ?: return@withLock outcome
            apply(snapshot = merged, edit = edit)
        }
    }

    private suspend fun apply(
        snapshot: LibrarySnapshotModel,
        edit: (CatalogModel) -> CatalogUpdate,
    ): OperationOutcome {
        return when (val update = edit(snapshot.catalog)) {
            is CatalogUpdate.Rejected -> OperationOutcome.Rejected(reason = update.reason)
            // An edit that changed nothing writes nothing.
            is CatalogUpdate.Applied -> if (update.catalog == snapshot.catalog) {
                OperationOutcome.Done(catalog = snapshot.catalog, pendingFolders = emptySet())
            } else {
                save(snapshot = snapshot, catalog = update.catalog)
            }
        }
    }

    private suspend fun save(snapshot: LibrarySnapshotModel, catalog: CatalogModel): OperationOutcome {
        val replication = replicator.saveCatalog(
            folders = snapshot.folders,
            catalog = catalog,
            basedOn = snapshot.catalog.revision,
        )
        return when (replication) {
            is Replication.PrimaryFailed -> OperationOutcome.StorageFailed(failure = replication.failure)
            is Replication.Done -> {
                state.value = catalog
                OperationOutcome.Done(catalog = catalog, pendingFolders = replication.pendingFolders)
            }
        }
    }

    /**
     * Reads the primary's catalog, merges it into the open one and publishes the result. `null` when it
     * cannot be read or belongs to another library; the refused save then stands as the outcome.
     */
    private suspend fun mergeWithPrimary(snapshot: LibrarySnapshotModel): LibrarySnapshotModel? {
        val load = catalogs.load(folder = snapshot.folders.primary) as? CatalogLoad.Found ?: return null
        if (load.catalog.libraryId != snapshot.catalog.libraryId) {
            return null
        }
        // The primary's copy is newer, so it wins where both differ. The merge carries at least its
        // revision, so the next save is accepted unless yet another device wrote in between.
        val merged = CatalogMerger.merge(first = load.catalog, second = snapshot.catalog).catalog
        state.value = merged
        return snapshot.copy(catalog = merged)
    }

    private fun OperationOutcome.isCatalogChanged(): Boolean {
        return this is OperationOutcome.StorageFailed && failure.reason == StorageFailure.CATALOG_CHANGED
    }
}
