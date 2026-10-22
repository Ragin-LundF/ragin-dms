package dms.application.port.storage

import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import kotlinx.coroutines.flow.Flow

/**
 * Tells when something changes below [path] in any of [folders], such as a file dropped into the
 * inbox. Only a hint to scan: the scan stays the source of truth. Folders that cannot be watched are
 * left out quietly.
 */
interface FolderWatcher {

    fun changes(folders: List<LibraryFolderModel>, path: RelativePathModel): Flow<Unit>
}
