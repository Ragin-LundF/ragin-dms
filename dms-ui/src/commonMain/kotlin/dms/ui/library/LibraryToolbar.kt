package dms.ui.library

import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.GridView
import androidx.compose.material.icons.rounded.TableRows
import androidx.compose.material3.Icon
import androidx.compose.material3.IconToggleButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import dms.ui.components.ActionButtons
import dms.ui.components.layout.ScreenHeader
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibraryUiState
import dms.ui.library.model.YearFilter
import dms.ui.library.search.LibrarySearch

/**
 * The selection's title and size, the search with its switch for the documents' text, the year
 * filter, the list or grid switch, and adding to the selected folder.
 */
@Composable
fun LibraryToolbar(
    state: LibraryUiState,
    busyReason: String?,
    onSearch: (String) -> Unit,
    onContentSearch: (Boolean) -> Unit,
    onYear: (YearFilter?) -> Unit,
    onLayout: (DocumentLayout) -> Unit,
    onAddFiles: () -> Unit,
    onNewFolder: () -> Unit,
) {
    val count = state.documents.size
    ScreenHeader(
        title = state.title,
        subtitle = if (count == 1) "1 document" else "$count documents",
    ) {
        LibrarySearch(
            query = state.query,
            search = state.contentSearch,
            onSearch = onSearch,
            onContentSearch = onContentSearch,
        )
        YearMenu(years = state.years, selected = state.year, onSelect = onYear)
        Surface(shape = MaterialTheme.shapes.medium, color = MaterialTheme.colorScheme.surfaceContainer) {
            Row {
                IconToggleButton(
                    checked = state.layout == DocumentLayout.LIST,
                    onCheckedChange = { onLayout(DocumentLayout.LIST) },
                ) { Icon(imageVector = Icons.Rounded.TableRows, contentDescription = "List") }
                IconToggleButton(
                    checked = state.layout == DocumentLayout.GRID,
                    onCheckedChange = { onLayout(DocumentLayout.GRID) },
                ) { Icon(imageVector = Icons.Rounded.GridView, contentDescription = "Grid") }
            }
        }
        ActionButtons(
            buttons = listOf(
                ActionButton(
                    label = "New folder",
                    icon = Icons.Rounded.CreateNewFolder,
                    style = ButtonStyle.TONAL,
                    onClick = onNewFolder,
                ),
                ActionButton(
                    label = "Add files",
                    icon = Icons.Rounded.Add,
                    style = ButtonStyle.FILLED,
                    onClick = onAddFiles,
                ),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
