package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CreateNewFolder
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.ScreenHeader
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.Spacing

/** The library folders of this device: one primary and any number of sync folders. */
@Composable
fun SettingsScreen(viewModel: SettingsViewModel) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(state = rememberScrollState())
            .padding(all = Spacing.xxlarge),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        ScreenHeader(title = "Settings", subtitle = "Where your library lives on this device")
        if (!state.configured) {
            WelcomeCard(busyReason = busyReason, onChoose = viewModel::choosePrimaryFolder)
        } else {
            AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
                CardTitle(
                    title = "Library folders",
                    icon = Icons.Rounded.Storage,
                    tint = MaterialTheme.colorScheme.secondary,
                    subtitle = "Every change is written to the primary first, then to each sync folder",
                )
                Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
                    state.rows.forEach { row ->
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
                        ActionButton(
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
    }
    DialogHost(controller = viewModel.dialogs)
}
