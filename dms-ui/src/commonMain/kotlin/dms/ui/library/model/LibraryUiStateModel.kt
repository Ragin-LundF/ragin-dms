package dms.ui.library.model

import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.smart.model.SmartFolderEntryModel

/**
 * The library screen, derived from the catalog and the user's choices. [busy] names the library
 * operation that runs now, [aiBusy] the AI run.
 */
data class LibraryUiStateModel(
    val isOpen: Boolean = false,
    val title: String = "All documents",
    val folders: List<FolderEntryModel> = emptyList(),
    val categories: List<CategoryEntryModel> = emptyList(),
    val uncategorizedCount: Int = 0,
    val tags: List<TagNameModel> = emptyList(),
    val tagCounts: Map<TagNameModel, Int> = emptyMap(),
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
    val query: String = "",
    val documents: List<DocumentRowModel> = emptyList(),
    val inspected: DocumentModel? = null,
    val busy: String? = null,
    val aiBusy: String? = null,
    val layout: DocumentLayout = DocumentLayout.LIST,
    val years: List<Int> = emptyList(),
    val year: YearFilterModel? = null,
    val collapsedGroups: Set<SidebarGroup> = emptySet(),
    val contentSearch: ContentSearchStateModel = ContentSearchStateModel(),
    val smartFolders: List<SmartFolderEntryModel> = emptyList(),
)
