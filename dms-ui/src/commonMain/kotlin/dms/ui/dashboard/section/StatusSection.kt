package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.panel.ActivityPanel
import dms.ui.dashboard.panel.FolderPanel
import dms.ui.theme.Spacing

/** The state of the library folders beside what the tool did lately. */
@Composable
fun StatusSection(state: DashboardUiStateModel) {
    Row(
        modifier = Modifier.fillMaxWidth().height(height = 340.dp),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        FolderPanel(folders = state.folders, modifier = Modifier.weight(weight = 1f))
        ActivityPanel(entries = state.activity, modifier = Modifier.weight(weight = 1f))
    }
}
