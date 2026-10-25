package dms.ui.library.filter.taxonomy

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.MoreHoriz
import androidx.compose.runtime.Composable
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.library.LibraryViewModel
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FacetOptionModel
import dms.ui.library.model.LibraryUiStateModel

/**
 * The ⋯ menu of a category or tag in its facet popover: since the sidebar has no trees any more,
 * categories and tags are created, renamed and deleted here. Other values have no menu.
 */
@Composable
fun TaxonomyMenu(
    option: FacetOptionModel,
    state: LibraryUiStateModel,
    busyReason: String?,
    viewModel: LibraryViewModel,
) {
    val actions = when (val filter = option.filter) {
        is FacetFilter.Category -> categoryActions(filter = filter, state = state, viewModel = viewModel)
        is FacetFilter.Tag -> listOf(
            ActionButtonModel(label = "Rename", icon = Icons.Rounded.Edit) {
                viewModel.taxonomy.renameTag(tag = filter.tag)
            },
            ActionButtonModel(label = "Remove", icon = Icons.Rounded.Delete) {
                viewModel.taxonomy.removeTag(tag = filter.tag)
            },
        )

        is FacetFilter.Year,
        is FacetFilter.Correspondent,
        is FacetFilter.Type,
        is FacetFilter.Folder,
        -> emptyList()
    }
    if (actions.isEmpty()) {
        return
    }
    ActionMenu(
        actions = actions,
        contentDescription = "Actions for ${option.label}",
        enabled = busyReason == null,
        busyReason = busyReason,
        icon = Icons.Rounded.MoreHoriz,
    )
}

private fun categoryActions(
    filter: FacetFilter.Category,
    state: LibraryUiStateModel,
    viewModel: LibraryViewModel,
): List<ActionButtonModel> {
    val entry = state.categories.firstOrNull { it.category.id == filter.categoryId } ?: return emptyList()
    val category = entry.category
    return listOf(
        ActionButtonModel(label = "Add subcategory", icon = Icons.Rounded.Add) {
            viewModel.taxonomy.newCategory(parentId = category.id)
        },
        ActionButtonModel(label = "Rename", icon = Icons.Rounded.Edit) {
            viewModel.taxonomy.renameCategory(category = category)
        },
        ActionButtonModel(
            label = "Delete",
            icon = Icons.Rounded.Delete,
            disabledReason = "Documents still use this category".takeIf { entry.documentCount > 0 },
        ) { viewModel.taxonomy.removeCategory(category = category) },
    )
}
