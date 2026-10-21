package dms.application.sync

import dms.application.common.ActivityRecorder
import dms.application.common.LibrarySnapshot
import dms.application.library.CatalogStore
import dms.application.library.OpenLibrary
import dms.application.library.model.OpenLibraryOutcome
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.sync.model.SyncAction
import dms.application.sync.model.SyncActionKind
import dms.application.sync.model.SyncOutcome
import dms.application.sync.model.SyncReport
import dms.domain.change.ChangeDetector
import dms.domain.change.DetectedChange
import dms.domain.change.FolderSnapshot
import dms.domain.library.LibraryFolder
import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/**
 * "Merge all folders": scans every reachable folder, including backups, and copies each file that is
 * missing somewhere into the folders that lack it, then merges and saves the catalogs.
 *
 * Never deletes and never overwrites. A path with different content in different folders is a
 * conflict for the user. A document moved outside the tool is left alone until the user confirms or
 * undoes that move: neither its new nor its old path is copied, so a move never becomes a duplicate.
 */
class MergeAllFolders(
    private val store: CatalogStore,
    private val scanner: LibraryScanner,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val openLibrary: OpenLibrary,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(): SyncOutcome {
        val library = store.current() ?: return SyncOutcome.LibraryNotOpen
        val snapshots = mutableMapOf<LibraryFolder, FolderSnapshot>()
        val unreachable = mutableSetOf<LibraryFolderId>()
        for (folder in library.folders.folders) {
            val scanned = scanner.scan(folder = folder, catalog = library.catalog)
            if (scanned is StorageResult.Success) {
                snapshots[folder] = scanned.value
            } else if (folder == library.folders.primary) {
                return SyncOutcome.PrimaryFailed(failure = scanned as StorageResult.Failure)
            } else {
                unreachable += folder.id
            }
        }
        val moves = ChangeDetector.detect(catalog = library.catalog, snapshots = snapshots.values.toList())
            .filterIsInstance<DetectedChange.MovedFile>()
        val paths = snapshots.values.flatMap { it.files.keys }.distinct().sortedBy { it.value }
        val files = paths.map { path ->
            mergePath(library = library, snapshots = snapshots, path = path, moves = moves)
        }
        val opened = openLibrary()
        activity.record(kind = ActivityKind.SYNCHRONIZED, subject = library.folders.primary.displayName)
        val total = files.fold(initial = SyncReport(unreachableFolders = unreachable)) { sum, report -> sum + report }
        return when (opened) {
            is OpenLibraryOutcome.Opened -> SyncOutcome.Completed(
                report = total + SyncReport(pendingFolders = opened.pendingFolders),
            )

            is OpenLibraryOutcome.PrimaryFailed -> SyncOutcome.PrimaryFailed(failure = opened.failure)
            OpenLibraryOutcome.NotConfigured -> SyncOutcome.LibraryNotOpen
        }
    }

    private suspend fun mergePath(
        library: LibrarySnapshot,
        snapshots: Map<LibraryFolder, FolderSnapshot>,
        path: RelativePath,
        moves: List<DetectedChange.MovedFile>,
    ): SyncReport {
        val holders = snapshots.filterValues { path in it.files }.keys
        // A folder where the user moved a document away from this path does not get it back.
        val movedAway = moves.filter { it.previousPath == path }.map { it.folderId }.toSet()
        val lacking = (snapshots.keys - holders).filterNot { it.id in movedAway }.toSet()
        if (lacking.isEmpty()) {
            return SyncReport()
        }
        if (moves.any { it.path == path }) {
            return SyncReport(leftAlone = holders.associate { it.id to listOf(path) })
        }
        val hashes = holders.map { folder ->
            FileComparison.effectiveHash(
                facts = snapshots.getValue(key = folder).files.getValue(key = path),
                path = path,
                catalog = library.catalog,
            )
        }
        if (hashes.any { it == null } || hashes.toSet().size > 1) {
            return SyncReport(conflicts = listOf(path))
        }
        return copyToLacking(library = library, path = path, source = holders.first(), lacking = lacking)
    }

    /** Copies into the primary first if it lacks the file, then from the primary into the sync folders. */
    private suspend fun copyToLacking(
        library: LibrarySnapshot,
        path: RelativePath,
        source: LibraryFolder,
        lacking: Set<LibraryFolder>,
    ): SyncReport {
        val primary = library.folders.primary
        val actions = mutableListOf<SyncAction>()
        if (primary in lacking) {
            val copied = storage.copyFile(from = source, to = primary, path = path, replaceExisting = false)
            if (copied is StorageResult.Failure) {
                return SyncReport(failedPaths = listOf(path))
            }
            actions += SyncAction(folderId = primary.id, kind = SyncActionKind.COPIED, path = path)
        }
        val targets = library.folders.syncFolders.filter { it in lacking }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = targets,
            operation = FolderOperation.CopyFromPrimary(path = path),
        )
        actions += targets.filter { it.id !in pending }.map {
            SyncAction(folderId = it.id, kind = SyncActionKind.COPIED, path = path)
        }
        return SyncReport(actions = actions, pendingFolders = pending)
    }
}
