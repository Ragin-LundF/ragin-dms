package dms.ui.library.sidebar

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiState
import dms.ui.library.sidebar.item.FolderItem
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.sidebar.model.ToggleTarget
import dms.ui.theme.Spacing

/** The folder section of the sidebar: the folder tree, which folds, and renaming the selected folder. */
fun LazyListScope.folderEntries(
    state: LibraryUiState,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val selection = state.selection
    val expanded = SidebarGroup.FOLDERS !in state.collapsedGroups
    item(key = "folders") {
        SidebarSection(
            title = "Folders",
            expanded = expanded,
            onToggle = { viewModel.toggle(target = ToggleTarget.Group(group = SidebarGroup.FOLDERS)) },
            addLabel = "New folder",
            busyReason = busyReason,
        ) { viewModel.newFolder() }
    }
    if (!expanded) {
        return
    }
    items(items = state.folders, key = { "folder:${it.path.value}" }) { entry ->
        FolderItem(
            entry = entry,
            selection = selection,
            drag = drag,
            onToggle = { viewModel.toggle(target = ToggleTarget.Folder(path = entry.path)) },
        ) { viewModel.select(selection = LibrarySelection.Folder(path = entry.path)) }
    }
    if (selection is LibrarySelection.Folder && !selection.path.isRoot) {
        item(key = "folder-actions") {
            ActionButtons(
                buttons = listOf(
                    ActionButton(label = "Rename folder", icon = Icons.Rounded.Edit, style = ButtonStyle.TEXT) {
                        viewModel.renameFolder(path = selection.path)
                    },
                ),
                enabled = busyReason == null,
                busyReason = busyReason,
                modifier = Modifier.padding(start = Spacing.large),
            )
        }
    }
}
