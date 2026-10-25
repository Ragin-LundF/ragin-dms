package dms.ui.library.selection

import androidx.compose.foundation.layout.Box
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.LibraryUiStateModel

/** "Tag" of the bulk bar: the tags by how often they are used, and a new one. */
@Composable
fun TagMenuButton(state: LibraryUiStateModel, disabledReason: String?, bulk: BulkActions) {
    var open by remember { mutableStateOf(value = false) }
    val tags = state.facets[Facet.TAG].orEmpty().mapNotNull { (it.filter as? FacetFilter.Tag)?.tag }
    Box {
        BarButton(label = "Tag", icon = Icons.Rounded.LocalOffer, disabledReason = disabledReason) { open = true }
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            tags.forEach { tag ->
                MenuChoice(text = tag.value, depth = 0) {
                    open = false
                    bulk.addTag(tag = tag)
                }
            }
            if (tags.isNotEmpty()) {
                HorizontalDivider()
            }
            DropdownMenuItem(
                text = { Text(text = "New tag…") },
                leadingIcon = { Icon(imageVector = Icons.Rounded.Add, contentDescription = null) },
                onClick = {
                    open = false
                    bulk.newTag()
                },
            )
        }
    }
}
