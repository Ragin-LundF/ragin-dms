package dms.ui.model

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AccountTree
import androidx.compose.material.icons.rounded.Checklist
import androidx.compose.material.icons.rounded.Dashboard
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.MoveToInbox
import androidx.compose.material.icons.rounded.QuestionAnswer
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.ui.graphics.vector.ImageVector

/** The main screens, in the order of the navigation, with their [group]; `null` is pinned to the bottom. */
enum class Screen(val label: String, val icon: ImageVector, val group: NavGroup?) {
    DASHBOARD(label = "Dashboard", icon = Icons.Rounded.Dashboard, group = NavGroup.HOME),
    INBOX(label = "Inbox", icon = Icons.Rounded.MoveToInbox, group = NavGroup.HOME),
    TASKS(label = "Open tasks", icon = Icons.Rounded.Checklist, group = NavGroup.HOME),
    LIBRARY(label = "Library", icon = Icons.Rounded.Folder, group = NavGroup.DOCUMENTS),
    ASK(label = "Ask", icon = Icons.Rounded.QuestionAnswer, group = NavGroup.DOCUMENTS),
    RESTRUCTURE(label = "Restructure", icon = Icons.Rounded.AccountTree, group = NavGroup.MAINTAIN),
    SETTINGS(label = "Settings", icon = Icons.Rounded.Settings, group = null),
}
