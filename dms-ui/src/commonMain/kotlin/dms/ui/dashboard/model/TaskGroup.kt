package dms.ui.dashboard.model

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.MergeType
import androidx.compose.material.icons.rounded.AccountTree
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.ContentCopy
import androidx.compose.material.icons.rounded.Edit
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.Event
import androidx.compose.material.icons.rounded.FindInPage
import androidx.compose.material.icons.rounded.Inventory2
import androidx.compose.material.icons.rounded.NewReleases
import androidx.compose.material.icons.rounded.Rule
import androidx.compose.material.icons.rounded.SwapHoriz
import androidx.compose.material.icons.rounded.SyncProblem
import androidx.compose.ui.graphics.vector.ImageVector

/** The kinds of open tasks, in the order the dashboard lists them. */
enum class TaskGroup(val label: String, val icon: ImageVector) {
    SUGGESTIONS(label = "Suggestions", icon = Icons.Rounded.AutoAwesome),
    DEADLINES(label = "Deadlines", icon = Icons.Rounded.Event),
    RETENTION(label = "Retention ended", icon = Icons.Rounded.Inventory2),
    LEARNED_RULES(label = "Learned rules", icon = Icons.Rounded.Rule),
    NEW_FILES(label = "New files", icon = Icons.Rounded.NewReleases),
    MISSING(label = "Missing files", icon = Icons.Rounded.FindInPage),
    MODIFIED(label = "Changed outside", icon = Icons.Rounded.Edit),
    MOVED(label = "Different names", icon = Icons.Rounded.SwapHoriz),
    DUPLICATES(label = "Duplicates", icon = Icons.Rounded.ContentCopy),
    CONFLICTS(label = "Conflicts", icon = Icons.Rounded.ErrorOutline),
    CATALOG_MERGE(label = "Catalog merge", icon = Icons.AutoMirrored.Rounded.MergeType),
    FAILED_OPERATIONS(label = "Failed operations", icon = Icons.Rounded.SyncProblem),
    RESTRUCTURE(label = "Folder structure", icon = Icons.Rounded.AccountTree),
}
