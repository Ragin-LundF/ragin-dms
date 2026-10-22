package dms.ui.library.sidebar

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.item.TagItem
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.sidebar.model.ToggleTarget

/** The tag section of the sidebar, which folds; each tag is a drop target. */
fun LazyListScope.tagEntries(
    state: LibraryUiStateModel,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val expanded = SidebarGroup.TAGS !in state.collapsedGroups
    item(key = "tags") {
        SidebarSection(
            title = "Tags",
            expanded = expanded,
            onToggle = { viewModel.toggle(target = ToggleTarget.Group(group = SidebarGroup.TAGS)) },
        )
    }
    if (!expanded) {
        return
    }
    items(items = state.tags, key = { "tag:${it.value}" }) { tag ->
        TagItem(
            tag = tag,
            count = state.tagCounts[tag] ?: 0,
            selection = state.selection,
            busyReason = busyReason,
            drag = drag,
            viewModel = viewModel,
        )
    }
}
