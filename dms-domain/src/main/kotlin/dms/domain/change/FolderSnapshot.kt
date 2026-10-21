package dms.domain.change

import dms.domain.library.LibraryFolderId
import dms.domain.library.RelativePath

/** All files of one reachable library folder at the time of a scan. */
data class FolderSnapshot(
    val folderId: LibraryFolderId,
    val files: Map<RelativePath, FileFacts>,
)
