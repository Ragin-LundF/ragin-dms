package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.DialogHost
import dms.ui.components.ReasonTooltip
import dms.ui.components.SectionHeader
import dms.ui.theme.Spacing

/** The library folders of this device: one primary and any number of sync folders. */
@Composable
fun SettingsScreen(viewModel: SettingsViewModel) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(all = Spacing.xlarge)
            .verticalScroll(state = rememberScrollState()),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Text(text = "Settings", style = MaterialTheme.typography.headlineSmall)
        SectionHeader(title = "Library folders")
        if (!state.configured) {
            Text(
                text = "Choose the folder that holds your documents. It becomes the primary folder: every change " +
                    "is written there first, then to every sync folder. Your files stay readable files on disk.",
                modifier = Modifier.widthIn(max = 640.dp),
            )
            ReasonTooltip(reason = busyReason) {
                Button(onClick = viewModel::choosePrimaryFolder, enabled = busyReason == null) {
                    Text(text = "Choose primary folder…")
                }
            }
        } else {
            state.rows.forEach { row ->
                FolderSettingsRow(
                    row = row,
                    busyReason = busyReason,
                    onMakePrimary = { viewModel.makePrimary(folder = row.folder) },
                    onRemove = { viewModel.removeSyncFolder(folder = row.folder) },
                )
            }
            ReasonTooltip(reason = busyReason) {
                Button(onClick = viewModel::addSyncFolder, enabled = busyReason == null) {
                    Text(text = "Add sync folder…")
                }
            }
        }
    }
    DialogHost(controller = viewModel.dialogs)
}
