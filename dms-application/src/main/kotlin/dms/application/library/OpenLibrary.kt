package dms.application.library

import dms.application.library.model.OpenLibraryOutcome
import dms.application.port.state.SettingsRepository
import dms.application.port.state.TaskStore
import dms.application.port.storage.CatalogRepository
import dms.application.port.storage.model.CatalogLoad
import dms.application.port.system.IdGenerator
import dms.application.sync.Replicator
import dms.application.sync.model.Replication
import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogMergeResult
import dms.domain.catalog.CatalogMerger
import dms.domain.catalog.LibraryId
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.LibraryFolders

/**
 * Opens the library: reads the catalog of every reachable folder, merges all copies (including conflict
 * copies created by sync clients), saves the merged catalog where it is missing or differs, and
 * publishes it.
 *
 * A library without any catalog starts empty. A folder holding another library's catalog is ignored
 * and reported, never overwritten.
 */
class OpenLibrary(
    private val settings: SettingsRepository,
    private val catalogs: CatalogRepository,
    private val replicator: Replicator,
    private val store: CatalogStore,
    private val tasks: TaskStore,
    private val ids: IdGenerator,
) {

    suspend operator fun invoke(): OpenLibraryOutcome {
        val folders = settings.loadLibraryFolders() ?: return OpenLibraryOutcome.NotConfigured
        val loads = folders.folders.associateWith { catalogs.load(folder = it) }
        val primaryLoad = loads.getValue(key = folders.primary)
        if (primaryLoad is CatalogLoad.Failed) {
            return OpenLibraryOutcome.PrimaryFailed(failure = primaryLoad.failure)
        }
        val found = loads.mapNotNull { (folder, load) -> (load as? CatalogLoad.Found)?.let { folder to it } }.toMap()
        val libraryId = libraryIdOf(folders = folders, found = found)
        val foreign = found.filterValues { it.catalog.libraryId != libraryId }.keys
        val own = found - foreign
        // A conflict copy of another library's catalog cannot be merged; it stays where it is.
        val copies = own.values.flatMap { load ->
            listOf(load.catalog) + load.conflictCopies.filter { it.libraryId == libraryId }
        }
        val merge = mergeAll(libraryId = libraryId, copies = copies)

        val outdated = loads.filter { (folder, load) ->
            folder !in foreign && isOutdated(load = load, merged = merge.catalog)
        }
        val pending = if (outdated.isEmpty()) {
            emptySet()
        } else {
            // The primary's id wins, so a foreign folder is always a sync folder and can simply be left out.
            val ownFolders = LibraryFolders(folders = folders.folders - foreign)
            when (val saved = replicator.saveCatalog(folders = ownFolders, catalog = merge.catalog)) {
                is Replication.PrimaryFailed -> return OpenLibraryOutcome.PrimaryFailed(failure = saved.failure)
                is Replication.Done -> saved.pendingFolders
            }
        }
        discardMergedConflictCopies(found = own, pending = pending)

        store.publish(catalog = merge.catalog, foreign = foreign.map { it.id }.toSet())
        tasks.replaceMergeConflicts(conflicts = merge.conflicts)
        return OpenLibraryOutcome.Opened(
            catalog = merge.catalog,
            mergeConflicts = merge.conflicts,
            unreachableFolders = loads.filterValues { it is CatalogLoad.Failed }.keys.map { it.id }.toSet(),
            foreignFolders = foreign.map { it.id }.toSet(),
            pendingFolders = pending,
        )
    }

    /** The primary's library id wins; without any catalog a new library begins. */
    private fun libraryIdOf(folders: LibraryFolders, found: Map<LibraryFolder, CatalogLoad.Found>): LibraryId {
        val catalog = found[folders.primary]?.catalog ?: found.values.firstOrNull()?.catalog
        return catalog?.libraryId ?: LibraryId(value = ids.newId())
    }

    /** Unreachable folders are left alone; they get the catalog with the next sync or retry. */
    private fun isOutdated(load: CatalogLoad, merged: Catalog): Boolean {
        return when (load) {
            CatalogLoad.NotFound -> true
            is CatalogLoad.Found -> load.catalog != merged || load.conflictCopies.isNotEmpty()
            is CatalogLoad.Failed -> false
        }
    }

    private fun mergeAll(libraryId: LibraryId, copies: List<Catalog>): CatalogMergeResult {
        val first = copies.firstOrNull() ?: Catalog.empty(libraryId = libraryId)
        val initial = CatalogMergeResult(catalog = first, conflicts = emptyList())
        return copies.drop(n = 1).fold(initial = initial) { merged, copy ->
            val result = CatalogMerger.merge(first = merged.catalog, second = copy)
            CatalogMergeResult(catalog = result.catalog, conflicts = merged.conflicts + result.conflicts)
        }
    }

    private suspend fun discardMergedConflictCopies(
        found: Map<LibraryFolder, CatalogLoad.Found>,
        pending: Set<LibraryFolderId>,
    ) {
        found.filter { (folder, load) -> load.conflictCopies.isNotEmpty() && folder.id !in pending }
            .keys
            .forEach { catalogs.discardConflictCopies(folder = it) }
    }
}
