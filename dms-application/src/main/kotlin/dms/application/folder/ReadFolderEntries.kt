package dms.application.folder

import dms.application.folder.model.FolderEntriesOutcome
import dms.application.library.CatalogStore
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.FolderEntriesModel
import dms.application.port.storage.model.StorageFailure
import dms.application.port.storage.model.StorageResult
import dms.domain.library.model.RelativePathModel

/**
 * Reads what really lies below a folder, in every reachable library folder together: the folders,
 * also empty ones the catalog cannot know, and every file, also files not in the catalog or only in a
 * sync folder. A folder missing in one library folder holds nothing there. Only reads.
 */
class ReadFolderEntries(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
) {

    suspend operator fun invoke(path: RelativePathModel = RelativePathModel.ROOT): FolderEntriesOutcome {
        val library = store.current() ?: return FolderEntriesOutcome.LibraryNotOpen
        val primary = storage.listEntries(folder = library.folders.primary, path = path)
        if (primary is StorageResult.Failure && primary.reason != StorageFailure.NOT_FOUND) {
            return FolderEntriesOutcome.StorageFailed(failure = primary)
        }
        val reached = listOf(primary) + library.folders.syncFolders.map {
            storage.listEntries(folder = it, path = path)
        }
        // An offline sync folder or one without this folder adds nothing; its files come with the next sync.
        val entries = reached.mapNotNull { (it as? StorageResult.Success)?.value }
        return FolderEntriesOutcome.Read(
            entries = FolderEntriesModel(
                folders = entries.flatMap { it.folders }.toSet(),
                files = entries.flatMap { it.files }.toSet(),
            ),
        )
    }
}
