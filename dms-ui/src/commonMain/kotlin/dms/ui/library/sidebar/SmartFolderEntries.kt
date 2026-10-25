package dms.ui.library.sidebar

import androidx.compose.runtime.Composable
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.item.SmartFolderItem
import dms.ui.library.sidebar.model.SidebarGroup

/**
 * The "Saved views" section: the catalog's smart folders under their new name; "+" adds the standard
 * ones the library does not have yet. The rail always shows them.
 */
@Composable
fun SmartFolderEntries(state: LibraryUiStateModel, folded: Boolean, busyReason: String?, viewModel: LibraryViewModel) {
    val expanded = SidebarGroup.SAVED_VIEWS !in state.collapsedGroups
    SidebarSection(
        title = "Saved views",
        expanded = expanded,
        folded = folded,
        onToggle = { viewModel.toggleSection(group = SidebarGroup.SAVED_VIEWS) },
        addLabel = "Add the standard saved views",
        busyReason = busyReason,
        onAdd = viewModel.smartFolders::addPresets,
    )
    if (!expanded && !folded) {
        return
    }
    state.smartFolders.forEach { entry ->
        val selection = LibrarySelection.SmartFolder(id = entry.id)
        SmartFolderItem(
            entry = entry,
            selected = state.selection == selection,
            folded = folded,
            busyReason = busyReason,
            smartFolders = viewModel.smartFolders,
            onExport = {
                viewModel.smartFolders.searchOf(id = entry.id)?.let { viewModel.export.openSmartFolder(search = it) }
            },
        ) { viewModel.select(selection = selection) }
    }
}
