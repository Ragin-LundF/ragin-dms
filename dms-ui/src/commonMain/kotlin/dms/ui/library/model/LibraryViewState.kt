package dms.ui.library.model

import dms.domain.document.DocumentId
import dms.domain.library.RelativePath

/**
 * What the user chose on the library screen. [createdFolders] are empty folders created in this
 * session; the catalog only knows folders that contain documents.
 */
data class LibraryViewState(
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePath.ROOT),
    val query: String = "",
    val inspected: DocumentId? = null,
    val createdFolders: Set<RelativePath> = emptySet(),
    val layout: DocumentLayout = DocumentLayout.LIST,
)
