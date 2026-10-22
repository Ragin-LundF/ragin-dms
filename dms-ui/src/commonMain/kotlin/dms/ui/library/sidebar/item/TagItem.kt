package dms.ui.library.sidebar.item

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.runtime.Composable
import dms.domain.taxonomy.TagName
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButton
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.LibrarySelection

/** One tag; documents dropped on it get it. Its 3-dot menu renames or removes it. */
@Composable
fun TagItem(
    tag: TagName,
    count: Int,
    selection: LibrarySelection,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val actions = listOf(
        ActionButton(label = "Rename", icon = Icons.Rounded.Edit) { viewModel.renameTag(tag = tag) },
        ActionButton(label = "Remove", icon = Icons.Rounded.Delete) { viewModel.removeTag(tag = tag) },
    )
    DropTargetArea(drag = drag, target = DropTarget.Tag(tag = tag)) { hovered ->
        SidebarItem(
            label = tag.value,
            icon = Icons.Rounded.LocalOffer,
            count = count,
            depth = 0,
            selected = selection == LibrarySelection.Tag(tag = tag),
            dropHovered = hovered,
            menu = {
                ActionMenu(
                    actions = actions,
                    contentDescription = "Actions for ${tag.value}",
                    enabled = busyReason == null,
                    busyReason = busyReason,
                )
            },
        ) { viewModel.select(selection = LibrarySelection.Tag(tag = tag)) }
    }
}
