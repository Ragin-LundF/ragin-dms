package dms.ui.dashboard

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.FolderSpecial
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.components.surface.IconBadge
import dms.ui.dashboard.model.FolderHealth
import dms.ui.dashboard.model.FolderStatus
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/** The library folders with their role and whether they can be reached. */
@Composable
fun FolderPanel(folders: List<FolderStatus>, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    AppCard(modifier = modifier.fillMaxWidth()) {
        CardTitle(title = "Library folders", icon = Icons.Rounded.Storage, tint = colors.secondary)
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
            folders.forEach { folder ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
                ) {
                    IconBadge(
                        icon = if (folder.isPrimary) Icons.Rounded.FolderSpecial else Icons.Rounded.Folder,
                        tint = if (folder.isPrimary) colors.primary else colors.secondary,
                        size = 32.dp,
                    )
                    Column(modifier = Modifier.weight(weight = 1f)) {
                        Text(
                            text = folder.name + if (folder.isPrimary) " · primary" else "",
                            style = MaterialTheme.typography.titleSmall,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = folder.location,
                            style = MaterialTheme.typography.bodySmall,
                            color = colors.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    val healthColor = when (folder.health) {
                        FolderHealth.REACHABLE -> AppColors.statusGood
                        FolderHealth.OFFLINE -> colors.error
                        FolderHealth.FOREIGN -> AppColors.statusWarning
                        FolderHealth.NOT_SCANNED -> colors.outline
                    }
                    Box(modifier = Modifier.size(size = 8.dp).background(color = healthColor, shape = CircleShape))
                    val pending = if (folder.pendingOperations > 0) ", ${folder.pendingOperations} pending" else ""
                    Text(
                        text = folder.health.label + pending,
                        style = MaterialTheme.typography.labelSmall,
                        color = colors.onSurfaceVariant,
                        maxLines = 1,
                        softWrap = false,
                    )
                }
            }
        }
    }
}
