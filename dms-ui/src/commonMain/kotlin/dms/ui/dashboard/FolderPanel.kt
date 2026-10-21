package dms.ui.dashboard

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.SectionHeader
import dms.ui.dashboard.model.FolderHealth
import dms.ui.dashboard.model.FolderStatus
import dms.ui.theme.Spacing

/** The library folders with their status, and the size of the library. */
@Composable
fun FolderPanel(
    folders: List<FolderStatus>,
    documentCount: Int,
    uncategorizedCount: Int,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.padding(end = Spacing.large).verticalScroll(state = rememberScrollState())) {
        SectionHeader(title = "Folders")
        folders.forEach { folder ->
            Column(modifier = Modifier.padding(bottom = Spacing.small)) {
                val marker = if (folder.isPrimary) "★" else "●"
                Text(text = "$marker ${folder.name}", maxLines = 1, softWrap = false, overflow = TextOverflow.Ellipsis)
                Text(
                    text = folder.location,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
                val pending = if (folder.pendingOperations > 0) " · ${folder.pendingOperations} pending" else ""
                Text(
                    text = folder.health.label + pending,
                    style = MaterialTheme.typography.bodySmall,
                    color = if (folder.health == FolderHealth.REACHABLE || folder.health == FolderHealth.NOT_SCANNED) {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    } else {
                        MaterialTheme.colorScheme.error
                    },
                )
            }
        }
        SectionHeader(title = "Library")
        Text(text = "$documentCount documents")
        Text(text = "$uncategorizedCount uncategorized")
    }
}
