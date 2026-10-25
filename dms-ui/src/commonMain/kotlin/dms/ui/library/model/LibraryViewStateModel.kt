package dms.ui.library.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.ui.library.documents.model.DocumentSortModel
import dms.ui.library.documents.model.GroupKey
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.sidebar.model.SidebarGroup

/**
 * What the user chose on the library screen: the sidebar's view, the facet [filters] (at most one per
 * facet, in the order they were added) and the search text. Without a [sort], each group keeps the
 * filter's order.
 */
data class LibraryViewStateModel(
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
    val filters: List<FacetFilter> = emptyList(),
    val query: String = "",
    val inspected: DocumentIdModel? = null,
    val layout: DocumentLayout = DocumentLayout.GRID,
    val groupBy: GroupKey = GroupKey.MONTH,
    val collapsedGroups: Set<SidebarGroup> = emptySet(),
    val sort: DocumentSortModel? = null,
)
