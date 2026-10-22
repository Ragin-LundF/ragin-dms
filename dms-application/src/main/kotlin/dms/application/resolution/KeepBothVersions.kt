package dms.application.resolution

import dms.application.common.ActivityRecorder
import dms.application.common.OperationOutcome
import dms.application.common.SiblingNames
import dms.application.library.CatalogStore
import dms.application.port.state.model.ActivityKind
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderOperation
import dms.application.port.storage.model.StorageResult
import dms.application.sync.Replicator
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel

/**
 * Keeps two versions of a path: the copy in one folder is renamed to a free name with a
 * ` (2)` suffix and copied into every folder. The renamed file is not catalogued yet; the next scan
 * offers to adopt it.
 */
class KeepBothVersions(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
    private val replicator: Replicator,
    private val names: SiblingNames,
    private val activity: ActivityRecorder,
) {

    suspend operator fun invoke(path: RelativePathModel, folderId: LibraryFolderIdModel): OperationOutcome {
        val library = store.current() ?: return OperationOutcome.LibraryNotOpen
        val folder = library.folders.find(id = folderId) ?: return OperationOutcome.NotFound
        val parent = path.parent ?: RelativePathModel.ROOT
        val known = names.of(library = library, parent = parent)
        if (known is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = known)
        }
        val inFolders = library.folders.folders.flatMap { other ->
            (storage.listNames(folder = other, path = parent) as? StorageResult.Success)?.value.orEmpty()
        }
        val siblings = (known as StorageResult.Success).value + inFolders
        val newPath = parent.resolve(name = FileNamePolicy.nextFreeName(name = path.fileName, siblings = siblings))
        val renamed = storage.move(folder = folder, from = path, to = newPath)
        if (renamed is StorageResult.Failure) {
            return OperationOutcome.StorageFailed(failure = renamed)
        }
        val primary = library.folders.primary
        if (folder != primary) {
            val copied = storage.copyFile(from = folder, to = primary, path = newPath, replaceExisting = false)
            if (copied is StorageResult.Failure) {
                return OperationOutcome.StorageFailed(failure = copied)
            }
        }
        val pending = replicator.distribute(
            folders = library.folders,
            targets = library.folders.syncFolders.filter { it != folder },
            operation = FolderOperation.CopyFromPrimary(path = newPath),
        )
        activity.record(kind = ActivityKind.CONFLICT_RESOLVED, subject = newPath.value)
        return OperationOutcome.Done(catalog = library.catalog, pendingFolders = pending)
    }
}
