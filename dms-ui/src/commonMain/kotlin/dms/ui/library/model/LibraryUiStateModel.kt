package dms.ui.library.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.documents.model.CategoryTileModel
import dms.ui.library.documents.model.DocumentGroupModel
import dms.ui.library.documents.model.DocumentSortModel
import dms.ui.library.documents.model.GroupKey
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FacetOptionModel
import dms.ui.library.filter.model.FilterTokenModel
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.library.sidebar.model.SidebarCountsModel
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.smart.model.SmartFolderEntryModel

/**
 * The library screen, derived from the catalog and the user's choices. [busy] names the library
 * operation that runs now, [aiBusy] the AI run. [tokens] render [filters] in the search field, together
 * with the saved view's own conditions; [facets] are the values each facet chip offers, with counts.
 * [documents] are shown in [groups]; [selectedDocuments] are the ones ticked among them.
 * [queryProblem] says why the view cannot be saved or exported as a query, if it cannot.
 * [primaryFolderId] is where the thumbnails come from.
 */
data class LibraryUiStateModel(
    val isOpen: Boolean = false,
    val title: String = "All documents",
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
    val counts: SidebarCountsModel = SidebarCountsModel(),
    val categories: List<CategoryEntryModel> = emptyList(),
    val tags: List<TagNameModel> = emptyList(),
    val query: String = "",
    val filters: List<FacetFilter> = emptyList(),
    val tokens: List<FilterTokenModel> = emptyList(),
    val suggestions: List<FilterTokenModel> = emptyList(),
    val facets: Map<Facet, List<FacetOptionModel>> = emptyMap(),
    val tiles: List<CategoryTileModel> = emptyList(),
    val documents: List<DocumentRowModel> = emptyList(),
    val groups: List<DocumentGroupModel> = emptyList(),
    val groupBy: GroupKey = GroupKey.MONTH,
    val selectedDocuments: Set<DocumentIdModel> = emptySet(),
    val inspected: DocumentModel? = null,
    val busy: String? = null,
    val aiBusy: String? = null,
    /** Whether AI is set up; without it the library offers no AI action. */
    val aiEnabled: Boolean = true,
    val layout: DocumentLayout = DocumentLayout.GRID,
    val collapsedGroups: Set<SidebarGroup> = emptySet(),
    val contentSearch: ContentSearchStateModel = ContentSearchStateModel(),
    val smartFolders: List<SmartFolderEntryModel> = emptyList(),
    val queryProblem: String? = null,
    val sort: DocumentSortModel? = null,
    val primaryFolderId: LibraryFolderIdModel? = null,
) {
    /** `true` while a facet filter or search text narrows the documents. */
    val filtered: Boolean
        get() {
            return filters.isNotEmpty() || query.isNotBlank()
        }
}
