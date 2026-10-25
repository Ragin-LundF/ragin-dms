package dms.ui.shell.palette

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import dms.ui.shell.palette.model.PaletteItemModel
import dms.ui.shell.palette.model.PaletteStateModel
import dms.ui.theme.Spacing

/**
 * The command palette over the window. ↑/↓ move, ↵ chooses, ⌘↵ (Ctrl+↵) asks the library, Esc closes;
 * [onChoose] runs what the chosen line stands for.
 */
@Composable
fun CommandPaletteDialog(palette: CommandPalette, onChoose: (PaletteItemModel) -> Unit) {
    val state by palette.state.collectAsState()
    if (!state.open) {
        return
    }
    val focus = remember { FocusRequester() }
    LaunchedEffect(key1 = Unit) { focus.requestFocus() }
    Dialog(onDismissRequest = palette::close, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Surface(
            shape = MaterialTheme.shapes.extraLarge,
            color = MaterialTheme.colorScheme.surface,
            modifier = Modifier.width(width = Spacing.paletteWidth),
        ) {
            Column(modifier = Modifier.padding(all = Spacing.medium)) {
                OutlinedTextField(
                    value = state.query,
                    onValueChange = palette::type,
                    placeholder = { Text(text = "Search documents, tags, actions…") },
                    leadingIcon = { Icon(imageVector = Icons.Rounded.Search, contentDescription = null) },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .focusRequester(focusRequester = focus)
                        .onPreviewKeyEvent { event ->
                            handleKey(event = event, palette = palette, onChoose = onChoose)
                        },
                )
                PaletteList(
                    state = state,
                    onChoose = { item ->
                        palette.close()
                        onChoose(item)
                    },
                )
            }
        }
    }
}

@Composable
private fun PaletteList(state: PaletteStateModel, onChoose: (PaletteItemModel) -> Unit) {
    val listState = rememberLazyListState()
    val selected = state.items.getOrNull(index = state.selected)
    LaunchedEffect(key1 = state.selected, key2 = state.groups) {
        if (state.items.isNotEmpty()) {
            listState.animateScrollToItem(index = rowIndexOf(state = state))
        }
    }
    if (state.items.isEmpty()) {
        Text(
            text = "Nothing found",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(all = Spacing.large),
        )
        return
    }
    LazyColumn(
        state = listState,
        modifier = Modifier.heightIn(max = Spacing.paletteMaxHeight).padding(top = Spacing.small),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
    ) {
        state.groups.forEachIndexed { index, group ->
            item(key = "title-${group.title}") {
                if (index > 0) {
                    HorizontalDivider(modifier = Modifier.padding(vertical = Spacing.xsmall))
                }
                Text(
                    text = group.title,
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.xsmall),
                )
            }
            group.items.forEach { item ->
                item { PaletteRow(item = item, selected = item == selected, onClick = { onChoose(item) }) }
            }
        }
    }
}

/** The position of the selected line in the list, counting the title above every group. */
private fun rowIndexOf(state: PaletteStateModel): Int {
    var before = 0
    state.groups.forEachIndexed { index, group ->
        if (state.selected < before + group.items.size) {
            return before + index + 1 + (state.selected - before)
        }
        before += group.items.size
    }
    return 0
}

/** The keys of the palette; `true` when the key was used. */
private fun handleKey(event: KeyEvent, palette: CommandPalette, onChoose: (PaletteItemModel) -> Unit): Boolean {
    if (event.type != KeyEventType.KeyDown) {
        return false
    }
    val command = event.isMetaPressed || event.isCtrlPressed
    when (event.key) {
        Key.DirectionDown -> palette.move(steps = 1)
        Key.DirectionUp -> palette.move(steps = -1)
        Key.Escape -> palette.close()
        Key.Enter, Key.NumPadEnter -> (if (command) palette.ask() else palette.choose())?.let(onChoose)
        else -> return false
    }
    return true
}
