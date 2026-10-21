package dms.ui.library.sidebar.item

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AllInbox
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.runtime.Composable
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.FolderEntry
import dms.ui.library.model.LibrarySelection
import dms.ui.library.sidebar.model.TreeToggle

/** One folder of the folder tree, with a +/- for its subfolders; documents dropped on it move into it. */
@Composable
fun FolderItem(
    entry: FolderEntry,
    selection: LibrarySelection,
    drag: DocumentDrag,
    onToggle: () -> Unit,
    onSelect: () -> Unit,
) {
    val selected = selection == LibrarySelection.Folder(path = entry.path)
    val icon = when {
        entry.path.isRoot -> Icons.Rounded.AllInbox
        selected -> Icons.Rounded.FolderOpen
        else -> Icons.Rounded.Folder
    }
    DropTargetArea(drag = drag, target = DropTarget.Folder(path = entry.path)) { hovered ->
        SidebarItem(
            label = entry.name,
            icon = icon,
            count = entry.documentCount,
            depth = entry.depth,
            selected = selected,
            dropHovered = hovered,
            toggle = TreeToggle(
                expanded = entry.expanded.takeIf { entry.hasChildren && !entry.path.isRoot },
                onToggle = onToggle,
            ),
            onClick = onSelect,
        )
    }
}
