package dms.ui.library.folders.view

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.unit.Dp
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.PlatformActions
import dms.ui.components.layout.ResizeHandle
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.filter.TokenSearchField
import dms.ui.library.folders.FolderBrowser
import dms.ui.library.folders.view.tree.FolderTreePane
import dms.ui.library.picker.FolderPickerDialog
import dms.ui.theme.Spacing

/**
 * The folder view: the folder tree on the left (resizable, hideable), and on the right the path of the
 * open folder, its actions, its subfolders and its documents, which [documents] draws. "Add files here"
 * goes to [onAddFiles], files dropped from the file manager to [onDropFiles]; both get the open folder.
 * A click on an empty spot saves a folder name being typed. The search field above searches the open folder
 * and takes the focus each time [searchFocusRequests] grows ("/").
 */
@Composable
fun FolderView(
    browser: FolderBrowser,
    drag: DocumentDrag,
    platform: PlatformActions,
    onAddFiles: (RelativePathModel) -> Unit,
    onDropFiles: (List<String>, RelativePathModel) -> Unit,
    searchFocusRequests: Int,
    modifier: Modifier = Modifier,
    documents: @Composable (List<DocumentModel>) -> Unit,
) {
    val state by browser.state.collectAsState()
    val focus = LocalFocusManager.current
    Row(modifier = modifier.pointerInput(Unit) { detectTapGestures(onTap = { focus.clearFocus() }) }) {
        if (!state.treeHidden) {
            FolderTreePane(
                state = state,
                browser = browser,
                drag = drag,
                modifier = Modifier.width(width = Dp(value = state.treeWidth.toFloat())).fillMaxHeight(),
            )
            ResizeHandle(
                onResize = browser.treeLayout::resize,
                onResizeEnd = browser.treeLayout::endResize,
                onDoubleClick = browser.treeLayout::resetWidth,
            )
        }
        Column(
            modifier = Modifier.weight(weight = 1f).fillMaxHeight().padding(all = Spacing.xlarge),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            TokenSearchField(
                query = state.query,
                tokens = emptyList(),
                suggestions = emptyList(),
                placeholder = state.searchPlaceholder,
                focusRequests = searchFocusRequests,
                onQuery = browser::search,
                onRemove = {},
                onTake = {},
            )
            FolderHeader(state = state, browser = browser, drag = drag, onAddFiles = onAddFiles)
            FolderTiles(state = state, browser = browser, drag = drag)
            FolderDocuments(
                state = state,
                platform = platform,
                onDropFiles = onDropFiles,
                modifier = Modifier.weight(weight = 1f).fillMaxWidth(),
                documents = documents,
            )
        }
    }
    FolderPickerDialog(picker = browser.editor.picker, foldersOf = browser::pickerRows)
}

