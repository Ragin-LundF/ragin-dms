package dms.ui.library.sidebar

import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.item.SmartFolderItem
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.sidebar.model.ToggleTarget

/** The smart folders of the sidebar; "+" adds the standard ones the library does not have yet. */
fun LazyListScope.smartFolderEntries(state: LibraryUiStateModel, busyReason: String?, viewModel: LibraryViewModel) {
    val expanded = SidebarGroup.SMART_FOLDERS !in state.collapsedGroups
    item(key = "smart-folders") {
        SidebarSection(
            title = "Smart folders",
            expanded = expanded,
            onToggle = { viewModel.toggle(target = ToggleTarget.Group(group = SidebarGroup.SMART_FOLDERS)) },
            addLabel = "Add the standard smart folders",
            busyReason = busyReason,
            onAdd = viewModel.smartFolders::addPresets,
        )
    }
    if (!expanded) {
        return
    }
    items(items = state.smartFolders, key = { "smart:${it.id.value}" }) { entry ->
        SmartFolderItem(
            entry = entry,
            selection = state.selection,
            busyReason = busyReason,
            smartFolders = viewModel.smartFolders,
            onExport = {
                viewModel.smartFolders.searchOf(id = entry.id)?.let { viewModel.export.openSmartFolder(search = it) }
            },
        ) { viewModel.select(selection = LibrarySelection.SmartFolder(id = entry.id)) }
    }
}
