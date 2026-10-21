package dms.ui.library.search

import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.layout.SearchField
import dms.ui.library.search.model.ContentSearchState

/** The library's search field, with the switch that makes it search the documents' text too. */
@Composable
fun LibrarySearch(
    query: String,
    search: ContentSearchState,
    onSearch: (String) -> Unit,
    onContentSearch: (Boolean) -> Unit,
) {
    SearchField(
        query = query,
        onQueryChange = onSearch,
        placeholder = if (search.enabled) {
            "Search names, titles, tags and text"
        } else {
            "Filter by name, title, correspondent or tag"
        },
        modifier = Modifier.width(width = 360.dp),
    )
    ContentSearchChip(state = search, onEnable = onContentSearch)
}
