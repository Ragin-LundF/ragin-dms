package dms.application.sync

import dms.application.common.ActivityRecorder
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.library.OpenLibrary
import dms.application.library.model.outcome.OpenLibraryOutcome
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.LibraryScanner
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.sync.model.SyncActionKind
import dms.application.sync.model.SyncActionModel
import dms.application.sync.model.SyncOutcome
import dms.application.sync.model.SyncReportModel
import dms.domain.change.ChangeDetector
import dms.domain.change.DetectedChange
import dms.domain.change.model.FolderSnapshotModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

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
        val snapshots = mutableMapOf<LibraryFolderModel, FolderSnapshotModel>()
        val unreachable = mutableSetOf<LibraryFolderIdModel>()
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
        val start = SyncReportModel(unreachableFolders = unreachable)
        val total = files.fold(initial = start) { sum, report -> sum + report }
        return when (opened) {
            is OpenLibraryOutcome.Opened -> SyncOutcome.Completed(
                report = total + SyncReportModel(pendingFolders = opened.pendingFolders),
            )

            is OpenLibraryOutcome.PrimaryFailed -> SyncOutcome.PrimaryFailed(failure = opened.failure)
            OpenLibraryOutcome.NotConfigured -> SyncOutcome.LibraryNotOpen
        }
    }

    private suspend fun mergePath(
        library: LibrarySnapshotModel,
        snapshots: Map<LibraryFolderModel, FolderSnapshotModel>,
        path: RelativePathModel,
        moves: List<DetectedChange.MovedFile>,
    ): SyncReportModel {
        val holders = snapshots.filterValues { path in it.files }.keys
        // A folder where the user moved a document away from this path does not get it back.
        val movedAway = moves.filter { it.previousPath == path }.map { it.folderId }.toSet()
        val lacking = (snapshots.keys - holders).filterNot { it.id in movedAway }.toSet()
        if (lacking.isEmpty()) {
            return SyncReportModel()
        }
        if (moves.any { it.path == path }) {
            return SyncReportModel(leftAlone = holders.associate { it.id to listOf(path) })
        }
        val hashes = holders.map { folder ->
            FileComparison.effectiveHash(
                facts = snapshots.getValue(key = folder).files.getValue(key = path),
                path = path,
                catalog = library.catalog,
            )
        }
        if (hashes.any { it == null } || hashes.toSet().size > 1) {
            return SyncReportModel(conflicts = listOf(path))
        }
        return copyToLacking(library = library, path = path, source = holders.first(), lacking = lacking)
    }

    /** Copies into the primary first if it lacks the file, then from the primary into the sync folders. */
    private suspend fun copyToLacking(
        library: LibrarySnapshotModel,
        path: RelativePathModel,
        source: LibraryFolderModel,
        lacking: Set<LibraryFolderModel>,
    ): SyncReportModel {
        val primary = library.folders.primary
        val actions = mutableListOf<SyncActionModel>()
        if (primary in lacking) {
            val copied = storage.copyFile(from = source, to = primary, path = path, replaceExisting = false)
            if (copied is StorageResult.Failure) {
                return SyncReportModel(failedPaths = listOf(path))
            }
            actions += SyncActionModel(folderId = primary.id, kind = SyncActionKind.COPIED, path = path)
        }
        val targets = library.folders.syncFolders.filter { it in lacking }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = targets,
            operation = FolderOperation.CopyFromPrimary(path = path),
        )
        actions += targets.filter { it.id !in pending }.map {
            SyncActionModel(folderId = it.id, kind = SyncActionKind.COPIED, path = path)
        }
        return SyncReportModel(actions = actions, pendingFolders = pending)
    }
}
