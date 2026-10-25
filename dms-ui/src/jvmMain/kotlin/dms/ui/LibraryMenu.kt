package dms.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.window.MenuBarScope
import dms.ui.library.LibraryCommands
import dms.ui.platform.HostOs

/**
 * The "Documents" menu: every key of the library's table as a menu entry. Only ⌘\ has a menu shortcut;
 * the others name their key in parentheses in the label, because a menu shortcut for "/", Space, ⌘A, Esc, ↵ or ⌫ would
 * take that key from the text fields.
 */
@Composable
fun MenuBarScope.LibraryMenu(commands: LibraryCommands) {
    val open = commands.available
    Menu(text = "Documents") {
        Item(
            text = "Show or hide the sidebar",
            enabled = open,
            shortcut = HostOs.shortcut(key = Key.Backslash),
            onClick = commands::toggleSidebar,
        )
        Item(text = "Search documents (/)", enabled = open, onClick = commands::focusSearch)
        Item(text = "Quick look (Space)", enabled = open, onClick = commands::toggleQuickLook)
        Separator()
        Item(text = "Select all documents (${HostOs.commandLabel}A)", enabled = open, onClick = commands::selectAll)
        Item(text = "Clear the selection (Esc)", enabled = open, onClick = commands::clearSelection)
        Separator()
        Item(text = "New folder", enabled = open, onClick = commands::newFolder)
        Item(text = "Rename the open folder (↵)", enabled = open, onClick = commands::renameFolder)
        Item(text = "Delete the open folder… (⌫)", enabled = open, onClick = commands::deleteFolder)
    }
}
