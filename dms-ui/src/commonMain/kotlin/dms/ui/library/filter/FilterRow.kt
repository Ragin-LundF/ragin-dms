package dms.ui.library.filter

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Archive
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.FilterAlt
import androidx.compose.material.icons.rounded.GridView
import androidx.compose.material.icons.rounded.TableRows
import androidx.compose.material3.Icon
import androidx.compose.material3.IconToggleButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.library.filter.model.ActiveFilter
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.theme.Spacing

/**
 * The active filters as removable chips with "+ Filter" behind them, and on the right the number of
 * documents, the list or grid switch, saving the filters as a smart folder (only while there are
 * any) and a menu with Export… and New folder.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun FilterRow(
    state: LibraryUiStateModel,
    busyReason: String?,
    onRemove: (ActiveFilter) -> Unit,
    onYear: (YearFilterModel?) -> Unit,
    onContentSearch: (Boolean) -> Unit,
    onLayout: (DocumentLayout) -> Unit,
    onSaveSmartFolder: () -> Unit,
    onNewFolder: () -> Unit,
    onExport: () -> Unit,
) {
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
            state.filters.forEach { filter ->
                RemovableFilterChip(filter = filter, searching = state.contentSearch.searching) { onRemove(filter) }
            }
            AddFilterMenu(
                years = state.years,
                selected = state.year,
                contentSearch = state.contentSearch.enabled,
                onYear = onYear,
                onContentSearch = onContentSearch,
            )
        }
        ViewControls(
            state = state,
            busyReason = busyReason,
            onLayout = onLayout,
            onSaveSmartFolder = onSaveSmartFolder,
            onNewFolder = onNewFolder,
            onExport = onExport,
        )
    }
}

/** The number of documents, the layout switch, saving as a smart folder while filtered, and the ⋯ menu. */
@Composable
private fun ViewControls(
    state: LibraryUiStateModel,
    busyReason: String?,
    onLayout: (DocumentLayout) -> Unit,
    onSaveSmartFolder: () -> Unit,
    onNewFolder: () -> Unit,
    onExport: () -> Unit,
) {
    val count = state.documents.size
    Text(
        text = if (count == 1) "1 document" else "$count documents",
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        softWrap = false,
    )
    LayoutSwitch(layout = state.layout, onLayout = onLayout)
    if (state.filters.isNotEmpty()) {
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(
                    label = "Save as smart folder",
                    icon = Icons.Rounded.FilterAlt,
                    style = ButtonStyle.OUTLINED,
                    onClick = onSaveSmartFolder,
                ),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
    ActionMenu(
        actions = listOf(
            ActionButtonModel(
                label = "Export…",
                icon = Icons.Rounded.Archive,
                disabledReason = "Nothing to export".takeIf { count == 0 },
                onClick = onExport,
            ),
            ActionButtonModel(label = "New folder", icon = Icons.Rounded.CreateNewFolder, onClick = onNewFolder),
        ),
        contentDescription = "More library actions",
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}

@Composable
private fun LayoutSwitch(layout: DocumentLayout, onLayout: (DocumentLayout) -> Unit) {
    Surface(shape = MaterialTheme.shapes.medium, color = MaterialTheme.colorScheme.surfaceContainer) {
        Row {
            IconToggleButton(
                checked = layout == DocumentLayout.LIST,
                onCheckedChange = { onLayout(DocumentLayout.LIST) },
            ) { Icon(imageVector = Icons.Rounded.TableRows, contentDescription = "List") }
            IconToggleButton(
                checked = layout == DocumentLayout.GRID,
                onCheckedChange = { onLayout(DocumentLayout.GRID) },
            ) { Icon(imageVector = Icons.Rounded.GridView, contentDescription = "Grid") }
        }
    }
}
