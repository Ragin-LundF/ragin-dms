package dms.ui.library.sidebar.model

import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel

/** What a +/- in the sidebar folds or unfolds. */
sealed interface ToggleTarget {

    data class Folder(val path: RelativePathModel) : ToggleTarget

    data class Category(val categoryId: CategoryIdModel) : ToggleTarget

    data class Group(val group: SidebarGroup) : ToggleTarget
}
