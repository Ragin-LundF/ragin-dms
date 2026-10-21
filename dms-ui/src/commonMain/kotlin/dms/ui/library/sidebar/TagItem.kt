package dms.ui.library.sidebar

import androidx.compose.runtime.Composable
import dms.domain.taxonomy.TagName
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection

/** One tag of the sidebar; while selected it offers to rename or remove it. */
@Composable
fun TagItem(
    tag: TagName,
    selection: LibrarySelection,
    busyReason: String?,
    viewModel: LibraryViewModel,
) {
    val selected = selection == LibrarySelection.Tag(tag = tag)
    SidebarItem(label = tag.value, count = null, depth = 0, selected = selected) {
        viewModel.select(selection = LibrarySelection.Tag(tag = tag))
    }
    if (selected) {
        ActionButtons(
            buttons = listOf(
                ActionButton(label = "Rename…") { viewModel.renameTag(tag = tag) },
                ActionButton(label = "Remove…") { viewModel.removeTag(tag = tag) },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
