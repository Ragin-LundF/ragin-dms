package dms.ui.library.sidebar.model

import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId

/** What a +/- in the sidebar folds or unfolds. */
sealed interface ToggleTarget {

    data class Folder(val path: RelativePath) : ToggleTarget

    data class Category(val categoryId: CategoryId) : ToggleTarget

    data class Group(val group: SidebarGroup) : ToggleTarget
}
