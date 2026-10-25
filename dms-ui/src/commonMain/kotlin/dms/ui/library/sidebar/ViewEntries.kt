package dms.ui.library.sidebar

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.CalendarMonth
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.GridView
import androidx.compose.material.icons.rounded.Inbox
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import dms.domain.library.model.RelativePathModel
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.item.SidebarItem
import dms.ui.library.sidebar.model.SidebarGroup

/**
 * The "Library" section: All documents, Inbox and Needs review with badges in the warning tint, Added
 * this month, and Folders, which opens the folder view. The rail always shows them.
 */
@Composable
fun ViewEntries(state: LibraryUiStateModel, folded: Boolean, viewModel: LibraryViewModel) {
    val expanded = SidebarGroup.LIBRARY !in state.collapsedGroups
    SidebarSection(
        title = "Library",
        expanded = expanded,
        folded = folded,
        onToggle = { viewModel.toggleSection(group = SidebarGroup.LIBRARY) },
    )
    if (!expanded && !folded) {
        return
    }
    val counts = state.counts
    listOf(
        View(
            selection = LibrarySelection.Folder(path = RelativePathModel.ROOT),
            label = "All documents",
            icon = Icons.Rounded.GridView,
            count = counts.all,
        ),
        View(
            selection = LibrarySelection.Inbox,
            label = "Inbox",
            icon = Icons.Rounded.Inbox,
            count = counts.inbox,
            warning = true,
        ),
        View(
            selection = LibrarySelection.NeedsReview,
            label = "Needs review",
            icon = Icons.Rounded.AutoAwesome,
            count = counts.needsReview,
            warning = true,
        ),
        View(
            selection = LibrarySelection.Recent,
            label = "Added this month",
            icon = Icons.Rounded.CalendarMonth,
            count = counts.addedThisMonth,
        ),
        View(
            selection = LibrarySelection.Folders,
            label = "Folders",
            icon = Icons.Rounded.Folder,
            count = counts.folders,
        ),
    ).forEach { view ->
        SidebarItem(
            label = view.label,
            icon = view.icon,
            count = view.count,
            selected = state.selection == view.selection,
            folded = folded,
            warning = view.warning,
        ) { viewModel.select(selection = view.selection) }
    }
}

/** One view of the "Library" section. */
private data class View(
    val selection: LibrarySelection,
    val label: String,
    val icon: ImageVector,
    val count: Int,
    val warning: Boolean = false,
)
