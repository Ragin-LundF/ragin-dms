package dms.ui.library.sidebar

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.library.RelativePath
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButton
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiState

/** Folders, categories and tags; the selected entry decides which documents the list shows. */
@Composable
fun LibrarySidebar(
    state: LibraryUiState,
    busyReason: String?,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    val enabled = busyReason == null
    val selection = state.selection
    LazyColumn(modifier = modifier) {
        item(key = "folders") { SectionHeader(title = "Folders") }
        items(items = state.folders, key = { "folder:${it.path.value}" }) { entry ->
            SidebarItem(
                label = entry.name,
                count = entry.documentCount,
                depth = entry.depth,
                selected = selection == LibrarySelection.Folder(path = entry.path),
            ) { viewModel.select(selection = LibrarySelection.Folder(path = entry.path)) }
        }
        if (selection is LibrarySelection.Folder && selection.path != RelativePath.ROOT) {
            item(key = "folder-actions") {
                ActionButtons(
                    buttons = listOf(
                        ActionButton(label = "Rename folder…") { viewModel.renameFolder(path = selection.path) },
                    ),
                    enabled = enabled,
                    busyReason = busyReason,
                )
            }
        }
        item(key = "categories") {
            SectionHeader(title = "Categories") {
                ActionButtons(
                    buttons = listOf(ActionButton(label = "New…", onClick = viewModel::newCategory)),
                    enabled = enabled,
                    busyReason = busyReason,
                )
            }
        }
        item(key = "uncategorized") {
            SidebarItem(
                label = "Uncategorized",
                count = state.uncategorizedCount,
                depth = 0,
                selected = selection == LibrarySelection.Uncategorized,
            ) { viewModel.select(selection = LibrarySelection.Uncategorized) }
        }
        items(items = state.categories, key = { "category:${it.category.id.value}" }) { entry ->
            CategoryItem(entry = entry, selection = selection, busyReason = busyReason, viewModel = viewModel)
        }
        item(key = "tags") { SectionHeader(title = "Tags") }
        items(items = state.tags, key = { "tag:${it.value}" }) { tag ->
            TagItem(tag = tag, selection = selection, busyReason = busyReason, viewModel = viewModel)
        }
    }
}
