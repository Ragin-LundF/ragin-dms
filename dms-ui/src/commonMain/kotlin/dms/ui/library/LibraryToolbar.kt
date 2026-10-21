package dms.ui.library

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.layout.ScreenHeader
import dms.ui.components.layout.SearchField
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibraryUiState
import dms.ui.library.model.YearFilter

/**
 * The selection's title and size, the text and year filters, the list or grid switch, and adding to
 * the selected folder.
 */
@Composable
fun LibraryToolbar(
    state: LibraryUiState,
    busyReason: String?,
    onSearch: (String) -> Unit,
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
        SearchField(
            query = state.query,
            onQueryChange = onSearch,
            placeholder = "Filter by name, title, correspondent or tag",
            modifier = Modifier.width(width = 360.dp),
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
