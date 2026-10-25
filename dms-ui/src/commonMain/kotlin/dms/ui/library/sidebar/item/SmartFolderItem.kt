package dms.ui.library.sidebar.item

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Archive
import androidx.compose.material.icons.rounded.Bolt
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Warning
import androidx.compose.runtime.Composable
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.library.smart.SmartFolders
import dms.ui.library.smart.model.SmartFolderEntryModel

/**
 * One saved view (a smart folder of the catalog); its 3-dot menu renames it, replaces its search with
 * the current view, exports it, or deletes it. One that names a category, tag or field that no longer
 * exists shows a warning.
 */
@Composable
fun SmartFolderItem(
    entry: SmartFolderEntryModel,
    selected: Boolean,
    folded: Boolean,
    busyReason: String?,
    smartFolders: SmartFolders,
    onExport: () -> Unit,
    onSelect: () -> Unit,
) {
    val actions = listOf(
        ActionButtonModel(label = "Rename", icon = Icons.Rounded.Edit) { smartFolders.rename(id = entry.id) },
        ActionButtonModel(label = "Use the current view", icon = Icons.Rounded.Refresh) {
            smartFolders.replaceWithCurrent(id = entry.id)
        },
        ActionButtonModel(label = "Export…", icon = Icons.Rounded.Archive, onClick = onExport),
        ActionButtonModel(label = "Delete", icon = Icons.Rounded.Delete) { smartFolders.delete(id = entry.id) },
    )
    SidebarItem(
        label = if (entry.invalid) "${entry.name} (invalid)" else entry.name,
        icon = if (entry.invalid) Icons.Rounded.Warning else Icons.Rounded.Bolt,
        count = entry.count,
        selected = selected,
        folded = folded,
        menu = {
            ActionMenu(
                actions = actions,
                contentDescription = "Actions for ${entry.name}",
                enabled = busyReason == null,
                busyReason = busyReason,
            )
        },
        onClick = onSelect,
    )
}
