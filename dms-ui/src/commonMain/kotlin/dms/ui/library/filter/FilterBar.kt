package dms.ui.library.filter

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Bolt
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.library.LibraryViewModel
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.taxonomy.NewCategoryButton
import dms.ui.library.filter.taxonomy.TaxonomyMenu
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.theme.Spacing

/**
 * Above the documents: the search field with the active filters as tokens, and below it the facet
 * chips, "Clear" and "Save view" while anything filters, and on the right the grouping, the grid/list
 * switch and the ⋯ menu. Tokens and chips render the same filters of the view model.
 */
@Composable
fun FilterBar(
    state: LibraryUiStateModel,
    busyReason: String?,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    val focusRequests by viewModel.searchFocusRequests.collectAsState()
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        TokenSearchField(
            query = state.query,
            tokens = state.tokens,
            suggestions = state.suggestions,
            placeholder = placeholderOf(state = state),
            focusRequests = focusRequests,
            onQuery = viewModel::search,
            onRemove = viewModel::removeFilter,
            onTake = { viewModel.addFilter(filter = it, clearQuery = true) },
        )
        FacetRow(state = state, busyReason = busyReason, viewModel = viewModel)
    }
}

/** The facet chips, Clear and Save view while anything filters, and the view controls on the right. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun FacetRow(state: LibraryUiStateModel, busyReason: String?, viewModel: LibraryViewModel) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        FlowRow(
            modifier = Modifier.weight(weight = 1f),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
            itemVerticalAlignment = Alignment.CenterVertically,
        ) {
            Facet.entries.forEach { facet ->
                FacetChip(
                    facet = facet,
                    state = state,
                    onPick = { viewModel.addFilter(filter = it) },
                    onRemove = viewModel::removeFilter,
                    optionMenu = { option ->
                        TaxonomyMenu(option = option, state = state, busyReason = busyReason, viewModel = viewModel)
                    },
                    footer = { close ->
                        NewCategoryButton(
                            facet = facet,
                            busyReason = busyReason,
                            viewModel = viewModel,
                            onDone = close,
                        )
                    },
                )
            }
            if (state.filtered) {
                TextButton(onClick = viewModel::clearFilters) { Text(text = "Clear") }
                ActionButtons(
                    buttons = listOf(
                        ActionButtonModel(
                            label = "Save view",
                            icon = Icons.Rounded.Bolt,
                            style = ButtonStyle.OUTLINED,
                            disabledReason = state.queryProblem,
                            onClick = viewModel.smartFolders::saveCurrent,
                        ),
                    ),
                    enabled = busyReason == null,
                    busyReason = busyReason,
                )
            }
        }
        ViewControls(state = state, busyReason = busyReason, viewModel = viewModel)
    }
}

private fun placeholderOf(state: LibraryUiStateModel): String {
    return when {
        state.tokens.isNotEmpty() -> "Add words…"
        state.contentSearch.enabled -> "Search text, names, tags… try “invoice” or “2025”"
        else -> "Search names, titles, correspondents and tags…"
    }
}
