package dms.ui.library

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/**
 * Lets the menu bar reach the open library, whose view model lives inside the library's screen. Every
 * entry does what its key does; without an open library the entries do nothing and are disabled.
 */
class LibraryCommands {

    private var library: LibraryViewModel? by mutableStateOf(value = null)

    /** Whether there is a library to act on. */
    val available: Boolean
        get() {
            return library != null
        }

    /** Connects the open library's [viewModel]; `null` disconnects it when the library closes. */
    fun connect(viewModel: LibraryViewModel?) {
        library = viewModel
    }

    fun toggleSidebar() {
        library?.sidebarWidth?.toggleFolded()
    }

    fun focusSearch() {
        library?.focusSearch()
    }

    fun toggleQuickLook() {
        library?.let { LibraryKeys.toggleQuickLook(viewModel = it) }
    }

    fun selectAll() {
        library?.documentSelection?.selectAll()
    }

    fun clearSelection() {
        library?.documentSelection?.clear()
    }

    fun newFolder() {
        library?.folders?.newFolder()
    }

    fun renameFolder() {
        library?.folders?.renameOpenFolder()
    }

    fun deleteFolder() {
        library?.folders?.deleteOpenFolder()
    }
}
