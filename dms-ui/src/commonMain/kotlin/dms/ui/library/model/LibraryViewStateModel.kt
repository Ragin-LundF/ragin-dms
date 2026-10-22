package dms.ui.library.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.ui.library.sidebar.model.SidebarGroup

/**
 * What the user chose on the library screen. [createdFolders] are empty folders created in this
 * session; the catalog only knows folders that contain documents. Subfolders and subcategories are
 * folded until the user unfolds them.
 */
data class LibraryViewStateModel(
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
    val query: String = "",
    val inspected: DocumentIdModel? = null,
    val createdFolders: Set<RelativePathModel> = emptySet(),
    val layout: DocumentLayout = DocumentLayout.LIST,
    val year: YearFilterModel? = null,
    val expandedFolders: Set<RelativePathModel> = emptySet(),
    val expandedCategories: Set<CategoryIdModel> = emptySet(),
    val collapsedGroups: Set<SidebarGroup> = emptySet(),
)
