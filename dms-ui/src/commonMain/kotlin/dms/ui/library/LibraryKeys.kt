package dms.ui.library

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel

/**
 * The library's keys outside the text fields, which take their own keys first, as the concept's table
 * names them. While quick look edits a value, every key belongs to that field.
 */
object LibraryKeys {

    /** `true` when [event] was one of the library's keys. */
    fun handle(event: KeyEvent, state: LibraryUiStateModel, viewModel: LibraryViewModel): Boolean {
        if (event.type != KeyEventType.KeyDown || viewModel.quickLook.state.value.editing != null) {
            return false
        }
        if (quickLookKey(event = event, viewModel = viewModel)) {
            return true
        }
        if (state.selection == LibrarySelection.Folders && folderKey(event = event, viewModel = viewModel)) {
            return true
        }
        return documentsKey(event = event, state = state, viewModel = viewModel)
    }

    /** Space opens and closes quick look, ↑/↓ move through the documents while it is open. */
    private fun quickLookKey(event: KeyEvent, viewModel: LibraryViewModel): Boolean {
        val open = viewModel.quickLook.isOpen
        when {
            event.key == Key.Spacebar && !event.isCommand() -> toggleQuickLook(viewModel = viewModel)
            event.key == Key.DirectionDown && open -> viewModel.quickLook.next()
            event.key == Key.DirectionUp && open -> viewModel.quickLook.previous()
            else -> return false
        }
        return true
    }

    /**
     * "/" searches, ⌘A ticks everything shown, Esc closes quick look and then clears the ticks; in the folder
     * view too, where "/" searches the open folder. An open popover takes Esc itself before the library sees it.
     */
    private fun documentsKey(event: KeyEvent, state: LibraryUiStateModel, viewModel: LibraryViewModel): Boolean {
        when {
            event.key == Key.Slash && !event.isCommand() -> viewModel.focusSearch()
            event.key == Key.A && event.isCommand() -> viewModel.documentSelection.selectAll()
            event.key == Key.Escape -> return escape(state = state, viewModel = viewModel)
            else -> return false
        }
        return true
    }

    /** In the folder view, ↵ renames the open folder and ⌫ deletes it after asking; the top level says why not. */
    private fun folderKey(event: KeyEvent, viewModel: LibraryViewModel): Boolean {
        when (event.key) {
            Key.Enter, Key.NumPadEnter -> viewModel.folders.renameOpenFolder()
            Key.Backspace, Key.Delete -> viewModel.folders.deleteOpenFolder()
            else -> return false
        }
        return true
    }

    private fun escape(state: LibraryUiStateModel, viewModel: LibraryViewModel): Boolean {
        if (viewModel.quickLook.close()) {
            return true
        }
        if (state.selectedDocuments.isEmpty()) {
            return false
        }
        viewModel.documentSelection.clear()
        return true
    }

    /** Space: closes quick look while it is open, or opens it on the first ticked document, else the first shown. */
    fun toggleQuickLook(viewModel: LibraryViewModel) {
        val current = viewModel.state.value
        val order = current.documents.map { it.document.id }
        val first = current.selectedDocuments.firstOrNull() ?: order.firstOrNull()
        viewModel.quickLook.toggle(documentId = first, order = order)
    }

    private fun KeyEvent.isCommand(): Boolean {
        return isMetaPressed || isCtrlPressed
    }
}
