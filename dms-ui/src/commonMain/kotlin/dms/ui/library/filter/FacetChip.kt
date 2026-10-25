package dms.ui.library.filter

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material.icons.rounded.CalendarToday
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.Description
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FacetOptionModel
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * One facet as a chip; while its filter is active it is tinted and shows the value of its token, so chip
 * and token always say the same. A click opens the popover with the facet's values; choosing one
 * applies it ([onPick]), choosing the active one removes it ([onRemove]). [optionMenu] and [footer]
 * add the category and tag actions.
 */
@Composable
fun FacetChip(
    facet: Facet,
    state: LibraryUiStateModel,
    onPick: (FacetFilter) -> Unit,
    onRemove: (FacetFilter) -> Unit,
    optionMenu: @Composable (FacetOptionModel) -> Unit = {},
    footer: @Composable (close: () -> Unit) -> Unit = {},
) {
    var open by remember { mutableStateOf(value = false) }
    val active = state.filters.firstOrNull { FilterTokens.facetOf(filter = it) == facet }
    val value = active?.let { filter -> state.tokens.firstOrNull { it.filter == filter }?.value }
    val colors = MaterialTheme.colorScheme
    Box {
        FilterChip(
            selected = active != null,
            onClick = { open = true },
            label = {
                Text(
                    text = if (value == null) facet.label else "${facet.label}: $value",
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
            },
            leadingIcon = {
                Icon(
                    imageVector = iconOf(facet = facet),
                    contentDescription = null,
                    modifier = Modifier.size(size = Spacing.large),
                )
            },
            trailingIcon = { Icon(imageVector = Icons.Rounded.ArrowDropDown, contentDescription = null) },
            shape = CircleShape,
            colors = FilterChipDefaults.filterChipColors(
                selectedContainerColor = colors.primaryContainer,
                selectedLabelColor = colors.onPrimaryContainer,
                selectedLeadingIconColor = colors.onPrimaryContainer,
                selectedTrailingIconColor = colors.onPrimaryContainer,
            ),
        )
        DropdownMenu(
            expanded = open,
            onDismissRequest = { open = false },
            modifier = Modifier.width(width = LibrarySizes.popoverWidth),
        ) {
            FacetPopover(
                facet = facet,
                options = state.facets[facet].orEmpty(),
                active = active,
                optionMenu = optionMenu,
                footer = { footer { open = false } },
            ) { picked ->
                open = false
                if (picked == active) onRemove(picked) else onPick(picked)
            }
        }
    }
}

private fun iconOf(facet: Facet): ImageVector {
    return when (facet) {
        Facet.CATEGORY -> Icons.Rounded.Category
        Facet.TAG -> Icons.Rounded.LocalOffer
        Facet.YEAR -> Icons.Rounded.CalendarToday
        Facet.CORRESPONDENT -> Icons.Rounded.Person
        Facet.TYPE -> Icons.Rounded.Description
        Facet.FOLDER -> Icons.Rounded.Folder
    }
}
