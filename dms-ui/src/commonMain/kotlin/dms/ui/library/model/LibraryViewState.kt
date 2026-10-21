package dms.ui.library.model

import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.ui.library.sidebar.model.SidebarGroup

/**
 * What the user chose on the library screen. [createdFolders] are empty folders created in this
 * session; the catalog only knows folders that contain documents. Subfolders and subcategories are
 * folded until the user unfolds them.
 */
data class LibraryViewState(
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePath.ROOT),
    val query: String = "",
    val inspected: DocumentId? = null,
    val createdFolders: Set<RelativePath> = emptySet(),
    val layout: DocumentLayout = DocumentLayout.LIST,
    val year: YearFilter? = null,
    val expandedFolders: Set<RelativePath> = emptySet(),
    val expandedCategories: Set<CategoryId> = emptySet(),
    val collapsedGroups: Set<SidebarGroup> = emptySet(),
)
