package dms.ui.settings.model

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.MoveToInbox
import androidx.compose.material.icons.rounded.Rule
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.TextFields
import androidx.compose.material.icons.rounded.TravelExplore
import androidx.compose.ui.graphics.vector.ImageVector

/**
 * The sections of a configured library's settings, in the order of the section list, each with the icon of
 * its card. A section that [needsLibrarySettings] is listed only once the library's settings are loaded.
 */
enum class SettingsSection(val label: String, val icon: ImageVector, val needsLibrarySettings: Boolean) {
    FOLDERS(label = "Library folders", icon = Icons.Rounded.Storage, needsLibrarySettings = false),
    SCANNING(label = "Scanning", icon = Icons.Rounded.TravelExplore, needsLibrarySettings = false),
    INBOX(label = "Inbox", icon = Icons.Rounded.MoveToInbox, needsLibrarySettings = true),
    FIELDS(label = "Custom fields", icon = Icons.Rounded.TextFields, needsLibrarySettings = true),
    RULES(label = "Matching rules", icon = Icons.Rounded.Rule, needsLibrarySettings = true),
    AI(label = "AI", icon = Icons.Rounded.AutoAwesome, needsLibrarySettings = false),
}
