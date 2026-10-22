package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Folder
import androidx.compose.material.icons.rounded.FolderSpecial
import androidx.compose.material.icons.rounded.Star
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.IconBadge
import dms.ui.components.surface.LabelChip
import dms.ui.settings.model.FolderRowModel
import dms.ui.theme.Spacing

/** One library folder with its role and the actions it allows. */
@Composable
fun FolderSettingsRow(row: FolderRowModel, busyReason: String?, onMakePrimary: () -> Unit, onRemove: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.medium,
        color = colors.surfaceContainerLow,
    ) {
        Row(
            modifier = Modifier.padding(all = Spacing.medium),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            IconBadge(
                icon = if (row.isPrimary) Icons.Rounded.FolderSpecial else Icons.Rounded.Folder,
                tint = if (row.isPrimary) colors.primary else colors.secondary,
            )
            Column(modifier = Modifier.weight(weight = 1f)) {
                Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
                    Text(text = row.folder.displayName, style = MaterialTheme.typography.titleSmall)
                    LabelChip(
                        text = if (row.isPrimary) "Primary" else "Sync folder",
                        color = if (row.isPrimary) colors.primary else colors.secondary,
                    )
                }
                Text(
                    text = row.folder.location.value,
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
                row.note?.let { note ->
                    Text(text = note, style = MaterialTheme.typography.bodySmall, color = colors.error)
                }
            }
            if (!row.isPrimary) {
                ActionButtons(
                    buttons = listOf(
                        ActionButtonModel(
                            label = "Make primary",
                            icon = Icons.Rounded.Star,
                            style = ButtonStyle.TEXT,
                            onClick = onMakePrimary,
                        ),
                        ActionButtonModel(
                            label = "Remove",
                            icon = Icons.Rounded.Delete,
                            style = ButtonStyle.TEXT,
                            onClick = onRemove,
                        ),
                    ),
                    enabled = busyReason == null,
                    busyReason = busyReason,
                )
            }
        }
    }
}
