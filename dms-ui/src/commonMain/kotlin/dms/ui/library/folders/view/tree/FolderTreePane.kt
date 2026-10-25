package dms.ui.library.folders.view.tree

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import dms.domain.library.model.RelativePathModel
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.folders.FolderBrowser
import dms.ui.library.folders.FolderContents
import dms.ui.library.folders.model.FolderBrowserStateModel
import dms.ui.library.folders.view.FolderMenu
import dms.ui.theme.Spacing

/**
 * The folder tree: "Library" (the top level) and its folders, with + for a new top-level folder and a
 * button that hides the tree.
 */
@Composable
fun FolderTreePane(
    state: FolderBrowserStateModel,
    browser: FolderBrowser,
    drag: DocumentDrag,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.padding(all = Spacing.small)) {
        Header(browser = browser)
        LazyColumn(verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline)) {
            item(key = "library") {
                FolderTreeRow(
                    path = RelativePathModel.ROOT,
                    label = FolderContents.LIBRARY_NAME,
                    icon = Icons.AutoMirrored.Rounded.LibraryBooks,
                    depth = 0,
                    count = state.libraryCount,
                    selected = state.open.isRoot,
                    expanded = null,
                    drag = drag,
                    onOpen = { browser.open(path = RelativePathModel.ROOT) },
                    onToggle = {},
                    onExpand = {},
                )
            }
            items(items = state.tree, key = { it.path.value }) { entry ->
                val selected = entry.path == state.open
                FolderTreeRow(
                    path = entry.path,
                    label = entry.name,
                    icon = if (selected) Icons.Rounded.FolderOpen else Icons.Rounded.Folder,
                    depth = entry.depth - 1,
                    count = entry.documentCount,
                    selected = selected,
                    expanded = entry.expanded.takeIf { entry.hasChildren },
                    drag = drag,
                    onOpen = { browser.open(path = entry.path) },
                    onToggle = { browser.toggle(path = entry.path) },
                    onExpand = { browser.expand(path = entry.path) },
                    menu = { FolderMenu(path = entry.path, browser = browser, busyReason = state.busyReason) },
                )
            }
        }
    }
}

@Composable
private fun Header(browser: FolderBrowser) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(start = Spacing.small, bottom = Spacing.xsmall),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = "Folders",
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.weight(weight = 1f),
        )
        IconButton(onClick = { browser.editor.newFolder(parent = RelativePathModel.ROOT) }) {
            Icon(
                imageVector = Icons.Rounded.Add,
                contentDescription = "New folder at the top level",
                modifier = Modifier.size(size = Spacing.icon),
            )
        }
        TreeToggleButton(hidden = false, onClick = browser.treeLayout::toggleFolded)
    }
}
