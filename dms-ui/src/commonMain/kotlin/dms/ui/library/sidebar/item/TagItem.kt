package dms.ui.library.sidebar.item

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.taxonomy.TagName
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.DropTargetArea
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.LibrarySelection
import dms.ui.theme.Spacing

/** One tag; documents dropped on it get it. While selected it offers to rename or remove it. */
@Composable
fun TagItem(
    tag: TagName,
    count: Int,
    selection: LibrarySelection,
    busyReason: String?,
    drag: DocumentDrag,
    viewModel: LibraryViewModel,
) {
    val selected = selection == LibrarySelection.Tag(tag = tag)
    DropTargetArea(drag = drag, target = DropTarget.Tag(tag = tag)) { hovered ->
        SidebarItem(
            label = tag.value,
            icon = Icons.Rounded.LocalOffer,
            count = count,
            depth = 0,
            selected = selected,
            dropHovered = hovered,
        ) { viewModel.select(selection = LibrarySelection.Tag(tag = tag)) }
    }
    if (selected) {
        ActionButtons(
            buttons = listOf(
                ActionButton(label = "Rename", icon = Icons.Rounded.Edit, style = ButtonStyle.TEXT) {
                    viewModel.renameTag(tag = tag)
                },
                ActionButton(label = "Remove", icon = Icons.Rounded.Delete, style = ButtonStyle.TEXT) {
                    viewModel.removeTag(tag = tag)
                },
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
            modifier = Modifier.padding(start = Spacing.large),
        )
    }
}
