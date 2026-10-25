package dms.ui.library.search

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.layout.SearchField
import dms.ui.library.search.model.ContentSearchStateModel

/**
 * The library's search field. Whether it searches the documents' text too is switched in the filter
 * row's "+ Filter" menu; the placeholder says which.
 */
@Composable
fun LibrarySearch(
    query: String,
    search: ContentSearchStateModel,
    onSearch: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    SearchField(
        query = query,
        onQueryChange = onSearch,
        placeholder = if (search.enabled) {
            "Search names, titles, tags and text"
        } else {
            "Filter by name, title, correspondent or tag"
        },
        modifier = modifier,
    )
}
