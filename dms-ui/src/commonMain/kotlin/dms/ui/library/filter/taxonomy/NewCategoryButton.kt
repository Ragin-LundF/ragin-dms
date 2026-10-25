package dms.ui.library.filter.taxonomy

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import dms.ui.components.ReasonTooltip
import dms.ui.library.LibraryViewModel
import dms.ui.library.filter.model.Facet

/** "New category" below the category popover's list; [onDone] closes the popover first. */
@Composable
fun NewCategoryButton(facet: Facet, busyReason: String?, viewModel: LibraryViewModel, onDone: () -> Unit) {
    if (facet != Facet.CATEGORY) {
        return
    }
    ReasonTooltip(reason = busyReason) {
        DropdownMenuItem(
            text = { Text(text = "New category…") },
            leadingIcon = { Icon(imageVector = Icons.Rounded.Add, contentDescription = null) },
            enabled = busyReason == null,
            onClick = {
                onDone()
                viewModel.taxonomy.newCategory(parentId = null)
            },
        )
    }
}
