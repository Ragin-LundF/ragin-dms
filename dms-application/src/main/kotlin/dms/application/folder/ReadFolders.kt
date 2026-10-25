package dms.application.folder

import dms.application.folder.model.FoldersOutcome
import dms.application.library.CatalogStore
import dms.application.port.storage.DocumentStorage
import dms.application.port.storage.model.StorageResult

/**
 * Reads the folders of the primary library folder, also empty ones the catalog cannot know, for the
 * folder tree and the pickers. Folders only, no files, and only the primary: the sync folders mirror it.
 * Only reads.
 */
class ReadFolders(
    private val store: CatalogStore,
    private val storage: DocumentStorage,
) {

    suspend operator fun invoke(): FoldersOutcome {
        val library = store.current() ?: return FoldersOutcome.LibraryNotOpen
        return when (val listed = storage.listFolders(folder = library.folders.primary)) {
            is StorageResult.Success -> FoldersOutcome.Read(folders = listed.value)
            is StorageResult.Failure -> FoldersOutcome.StorageFailed(failure = listed)
        }
    }
}
