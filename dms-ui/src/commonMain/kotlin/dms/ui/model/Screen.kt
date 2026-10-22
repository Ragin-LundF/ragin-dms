package dms.ui.model

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Checklist
import androidx.compose.material.icons.rounded.Dashboard
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.ui.graphics.vector.ImageVector

/** The main screens, in the order of the navigation. */
enum class Screen(val label: String, val icon: ImageVector) {
    DASHBOARD(label = "Dashboard", icon = Icons.Rounded.Dashboard),
    TASKS(label = "Open tasks", icon = Icons.Rounded.Checklist),
    LIBRARY(label = "Library", icon = Icons.Rounded.Folder),
    SETTINGS(label = "Settings", icon = Icons.Rounded.Settings),
}
