package dms.application.sync

import dms.application.common.ActivityRecorder
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
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
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel

/**
 * "Sync to all destinations": makes every reachable sync folder contain what the primary contains.
 *
 * Applies moves the catalog knows (a document still under its old path in the sync folder), copies
 * files the sync folder lacks, and saves the catalog there. Never deletes and never overwrites: files
 * only in the sync folder are left alone, files with different content are reported as conflicts.
 */
class SyncFromPrimary(
    private val store: CatalogStore,
    private val scanner: LibraryScanner,
    private val replicator: Replicator,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(): SyncOutcome {
        val library = store.current() ?: return SyncOutcome.LibraryNotOpen
        val primaryScan = scanner.scan(folder = library.folders.primary, catalog = library.catalog)
        if (primaryScan is StorageResult.Failure) {
            return SyncOutcome.PrimaryFailed(failure = primaryScan)
        }
        val source = (primaryScan as StorageResult.Success).value
        val report = library.folders.syncFolders
            .map { target -> syncFolder(library = library, source = source, target = target) }
            .fold(initial = SyncReportModel()) { total, folderReport -> total + folderReport }
        activity.record(kind = ActivityKind.SYNCHRONIZED, subject = library.folders.primary.displayName)
        return SyncOutcome.Completed(report = report)
    }

    private suspend fun syncFolder(
        library: LibrarySnapshotModel,
        source: FolderSnapshotModel,
        target: LibraryFolderModel,
    ): SyncReportModel {
        val scanned = scanner.scan(folder = target, catalog = library.catalog)
        if (scanned is StorageResult.Failure) {
            return SyncReportModel(unreachableFolders = setOf(target.id))
        }
        val snapshot = (scanned as StorageResult.Success).value
        val targetFiles = snapshot.files.toMutableMap()
        val moves = knownMoves(library = library, source = source, snapshot = snapshot).map { move ->
            val report = moveInTarget(library = library, target = target, move = move)
            if (report.pendingFolders.isEmpty()) {
                targetFiles.remove(key = move.path)?.let { targetFiles[move.previousPath] = it }
            }
            report
        }
        val copies = (source.files.keys - targetFiles.keys).sortedBy { it.value }.map { path ->
            copyToTarget(library = library, target = target, path = path)
        }
        val conflicts = source.files.filter { (path, facts) ->
            val other = targetFiles[path]
            other != null &&
                FileComparison.differs(first = facts, second = other, path = path, catalog = library.catalog)
        }.keys.sortedBy { it.value }
        val extra = (targetFiles.keys - source.files.keys).sortedBy { it.value }
        val catalogPending = replicator.distribute(
            folders = library.folders,
            targets = listOf(target),
            operation = FolderOperation.SaveCatalog,
        )
        val remaining = SyncReportModel(
            conflicts = conflicts,
            leftAlone = if (extra.isEmpty()) emptyMap() else mapOf(target.id to extra),
            pendingFolders = catalogPending,
        )
        return (moves + copies).fold(initial = remaining) { total, report -> total + report }
    }

    /** Documents the sync folder still holds under a path the primary no longer uses. */
    private fun knownMoves(
        library: LibrarySnapshotModel,
        source: FolderSnapshotModel,
        snapshot: FolderSnapshotModel,
    ): List<DetectedChange.MovedFile> {
        return ChangeDetector.detect(catalog = library.catalog, snapshots = listOf(snapshot))
            .filterIsInstance<DetectedChange.MovedFile>()
            .filter { it.previousPath in source.files }
    }

    private suspend fun moveInTarget(
        library: LibrarySnapshotModel,
        target: LibraryFolderModel,
        move: DetectedChange.MovedFile,
    ): SyncReportModel {
        val pending = replicator.distribute(
            folders = library.folders,
            targets = listOf(target),
            operation = FolderOperation.Move(from = move.path, to = move.previousPath),
        )
        if (pending.isNotEmpty()) {
            return SyncReportModel(pendingFolders = pending)
        }
        val action = SyncActionModel(folderId = target.id, kind = SyncActionKind.MOVED, path = move.previousPath)
        return SyncReportModel(actions = listOf(action))
    }

    private suspend fun copyToTarget(
        library: LibrarySnapshotModel,
        target: LibraryFolderModel,
        path: RelativePathModel,
    ): SyncReportModel {
        val pending = replicator.distribute(
            folders = library.folders,
            targets = listOf(target),
            operation = FolderOperation.CopyFromPrimary(path = path),
        )
        if (pending.isNotEmpty()) {
            return SyncReportModel(pendingFolders = pending)
        }
        val action = SyncActionModel(folderId = target.id, kind = SyncActionKind.COPIED, path = path)
        return SyncReportModel(actions = listOf(action))
    }
}
