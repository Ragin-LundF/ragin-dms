package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.settings.model.FolderRowModel
import dms.ui.theme.Spacing

/** The primary folder and the sync folders, with the actions that change them. */
@Composable
fun FoldersCard(rows: List<FolderRowModel>, busyReason: String?, viewModel: SettingsViewModel) {
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Library folders",
            icon = Icons.Rounded.Storage,
            tint = MaterialTheme.colorScheme.secondary,
            subtitle = "Every change is written to the primary first, then to each sync folder",
        )
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            rows.forEach { row ->
                FolderSettingsRow(
                    row = row,
                    busyReason = busyReason,
                    onMakePrimary = { viewModel.makePrimary(folder = row.folder) },
                    onRemove = { viewModel.removeSyncFolder(folder = row.folder) },
                )
            }
        }
        Text(
            text = "A sync folder can be any folder, such as one a cloud service keeps in sync.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(vertical = Spacing.medium),
        )
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(
                    label = "Add sync folder…",
                    icon = Icons.Rounded.CreateNewFolder,
                    style = ButtonStyle.TONAL,
                    onClick = viewModel::addSyncFolder,
                ),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
