package dms.ui.library.picker

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.LibraryBooks
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.window.DialogProperties
import dms.domain.library.model.RelativePathModel
import dms.ui.components.FoldToggle
import dms.ui.library.model.FolderEntryModel
import dms.ui.library.picker.model.FolderPickerStateModel
import dms.ui.theme.Spacing

/**
 * The picker of [picker] while it is open: the library's top level and its folders as a tree, from
 * [foldersOf], which gets the expanded folders and returns the visible rows.
 */
@Composable
fun FolderPickerDialog(picker: FolderPicker, foldersOf: (Set<RelativePathModel>) -> List<FolderEntryModel>) {
    val open by picker.state.collectAsState()
    val state = open ?: return
    AlertDialog(
        onDismissRequest = picker::dismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = Spacing.dialogWidth),
        title = { Text(text = state.title) },
        text = {
            LazyColumn(modifier = Modifier.heightIn(max = Spacing.paletteMaxHeight).fillMaxWidth()) {
                item { RootRow(state = state, onSelect = picker::select) }
                items(items = foldersOf(state.expanded), key = { it.path.value }) { folder ->
                    FolderRow(folder = folder, state = state, picker = picker)
                }
            }
        },
        confirmButton = {
            Button(onClick = picker::confirm, enabled = state.selected != null) { Text(text = state.confirmLabel) }
        },
        dismissButton = { TextButton(onClick = picker::dismiss) { Text(text = "Cancel") } },
    )
}

@Composable
private fun RootRow(state: FolderPickerStateModel, onSelect: (RelativePathModel) -> Unit) {
    PickerRow(
        label = "Library",
        depth = 0,
        selected = state.selected == RelativePathModel.ROOT,
        excluded = false,
        expanded = null,
        onToggle = {},
        root = true,
        onSelect = { onSelect(RelativePathModel.ROOT) },
    )
}

@Composable
private fun FolderRow(folder: FolderEntryModel, state: FolderPickerStateModel, picker: FolderPicker) {
    PickerRow(
        label = folder.name,
        depth = folder.depth + 1,
        selected = state.selected == folder.path,
        excluded = state.isExcluded(path = folder.path),
        expanded = folder.expanded.takeIf { folder.hasChildren },
        onToggle = { picker.toggle(path = folder.path) },
        root = false,
        onSelect = { picker.select(path = folder.path) },
    )
}

@Composable
private fun PickerRow(
    label: String,
    depth: Int,
    selected: Boolean,
    excluded: Boolean,
    expanded: Boolean?,
    onToggle: () -> Unit,
    root: Boolean,
    onSelect: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = MaterialTheme.shapes.small)
            .background(color = if (selected) colors.primaryContainer else Color.Transparent)
            .alpha(alpha = if (excluded) EXCLUDED_ALPHA else 1f)
            .clickable(enabled = !excluded, onClick = onSelect)
            .padding(
                start = Spacing.large * depth,
                top = Spacing.xsmall,
                bottom = Spacing.xsmall,
                end = Spacing.small,
            ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        FoldToggle(expanded = expanded, onToggle = onToggle)
        Icon(
            imageVector = if (root) Icons.Rounded.LibraryBooks else Icons.Rounded.Folder,
            contentDescription = null,
            tint = if (selected) colors.primary else colors.onSurfaceVariant,
            modifier = Modifier.size(size = Spacing.icon),
        )
        Text(
            text = label,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

private const val EXCLUDED_ALPHA = 0.4f
