package dms.ui.library.model

import dms.domain.document.Document
import dms.domain.library.RelativePath
import dms.domain.taxonomy.TagName
import dms.ui.library.sidebar.model.SidebarGroup

/** The library screen, derived from the catalog and the user's choices. */
data class LibraryUiState(
    val isOpen: Boolean = false,
    val title: String = "All documents",
    val folders: List<FolderEntry> = emptyList(),
    val categories: List<CategoryEntry> = emptyList(),
    val uncategorizedCount: Int = 0,
    val tags: List<TagName> = emptyList(),
    val tagCounts: Map<TagName, Int> = emptyMap(),
    val selection: LibrarySelection = LibrarySelection.Folder(path = RelativePath.ROOT),
    val query: String = "",
    val documents: List<DocumentRow> = emptyList(),
    val inspected: Document? = null,
    val busy: String? = null,
    val layout: DocumentLayout = DocumentLayout.LIST,
    val years: List<Int> = emptyList(),
    val year: YearFilter? = null,
    val collapsedGroups: Set<SidebarGroup> = emptySet(),
)
